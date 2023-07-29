use nalgebra::Affine3;
use nalgebra::Isometry3;
use nalgebra::Matrix3;
use nalgebra::Point2;
use nalgebra::Point3;
use nalgebra::Quaternion;
use nalgebra::Rotation3;
use nalgebra::Transform;
use nalgebra::Transform3;
use nalgebra::Translation3;
use nalgebra::Vector2;
use nalgebra::Vector3;
use observation_tools_client::builders::Image2Builder;
use observation_tools_client::builders::Object2Builder;
use observation_tools_client::builders::Object2Updater;
use observation_tools_client::builders::Object3Builder;
use observation_tools_client::builders::PerPixelTransformBuilder;
use observation_tools_client::builders::Polygon2Builder;
use observation_tools_client::builders::Polygon3Builder;
use observation_tools_client::builders::PolygonEdge3Builder;
use observation_tools_client::builders::SeriesBuilder;
use observation_tools_client::builders::SeriesPointBuilder;
use observation_tools_client::builders::Transform2Builder;
use observation_tools_client::builders::Transform3Builder;
use observation_tools_client::builders::Vector2Builder;
use observation_tools_client::ArtifactUploader2d;
use observation_tools_client::ArtifactUploader3d;
use observation_tools_client::GenericArtifactUploader;
use observation_tools_client::UserMetadataBuilder;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::error::Error;
use std::sync::Arc;
use tracing::info;

pub type GenericError = Box<dyn Error + Send + Sync>;

struct Stone {
    id: u8,
    grid_position: Point2<usize>,
    grid_size: Vector2<usize>,
}

struct AlgorithmParameters {
    desired_plank_width: f64,
    connection_points: Vec<f64>,
    max_stone_width: usize,
    max_stone_height: usize,
}

// Algorithm for procedurally generating a stone wall.
//
// NOTE:
// - The algorithm assumes that the input wall profile is perpendicular to the ground plane.
pub async fn generate_stone_wall(uploader_3d: ArtifactUploader3d) -> Result<(), GenericError> {
    let wall_profile_world = vec![
        Point3::new(108.0, 64.0, 13.0),
        Point3::new(108.0, 64.0, 15.0),
        Point3::new(111.0, 66.0, 16.0),
        Point3::new(114.0, 68.0, 15.0),
        Point3::new(114.0, 68.0, 13.0),
    ];
    let parameters = AlgorithmParameters {
        desired_plank_width: 1.0,
        connection_points: vec![0.0, 0.5, 1.5, 2.0, 2.5, 3.0],
        max_stone_width: 1,
        max_stone_height: 1,
    };

    uploader_3d
        .upload_object3(
            "wall_profile_world_space",
            Polygon3Builder::from_points(wall_profile_world.clone()),
        )
        .await?;
    let (world_to_local_transform, local_to_world_transform) =
        calculate_world_to_local_space_transforms(&wall_profile_world).await?;

    let wall_profile_2d: Vec<Point2<f64>> = wall_profile_world
        .iter()
        .map(|p| world_to_local_transform * p)
        .map(|p| Point2::new(p.x, p.y))
        .collect();

    let wall_2d_uploader = uploader_3d
        .child_uploader_2d("wall_2d", local_to_world_transform)
        .await?;

    wall_2d_uploader
        .upload_object2(
            "wall_profile_local_space",
            Polygon2Builder::from_points(wall_profile_2d.clone()),
        )
        .await?;

    let stones = generate_stone_locations(&parameters, &wall_2d_uploader).await?;

    // TODO(doug): Convert the grid pattern to boxes
    // TODO(doug): Apply adjustments to boxes to add randomness
    // TODO(doug): Mask the boxes to the wall profile

    Ok(())
}

async fn calculate_world_to_local_space_transforms(
    wall_profile_world: &Vec<Point3<f64>>,
) -> Result<(Transform3<f64>, Transform3<f64>), GenericError> {
    // Calculate the normal of the wall
    let side1 = wall_profile_world[2] - wall_profile_world[1];
    let side2 = wall_profile_world[0] - wall_profile_world[1];
    let global_y = side1.cross(&side2).normalize();
    let global_z = Vector3::z();
    let global_x = global_y.cross(&global_z).normalize();

    // Define a local coordinate system that aligns the face on the XZ plane (aka. zeroing the y coordinate)
    // TODO(doug): Zero the Z coordinate instead of the Y
    let origin = wall_profile_world[0];
    let local_to_world_transform = Isometry3::from_parts(
        origin.into(),
        Rotation3::rotation_between(&Vector3::x(), &global_x)
            .expect("Unable to calculate rotation between vectors")
            .into(),
    );
    let world_to_local_transform: Transform3<f64> =
        nalgebra::convert(local_to_world_transform.inverse());
    // TODO(doug): Remove need to convert to Transform3 manually
    let local_to_world_transform: Transform3<f64> = nalgebra::convert(local_to_world_transform);

    Ok((world_to_local_transform, local_to_world_transform))
}

async fn generate_stone_locations(
    parameters: &AlgorithmParameters,
    wall_2d_uploader: &ArtifactUploader2d,
) -> Result<Vec<Stone>, GenericError> {
    let mut series_builder = SeriesBuilder::new();
    let algorithm_step_dimension_id = series_builder.add_dimension("algorithm_step");
    let algorithm_series_id = wall_2d_uploader
        .series("grid_algorithm", series_builder)
        .await?;

    // TODO(doug): Get width from polygon
    let width = 3.0;
    let height = 3.0;
    let plank_column_count = (width / parameters.desired_plank_width);
    let plank_column_count = plank_column_count.floor() as usize;

    let mut rng = ChaCha8Rng::seed_from_u64(2);
    let grid_width = parameters.connection_points.len() - 1;
    let grid_height = plank_column_count;
    let mut grid = vec![0u8; grid_width * grid_height];
    let grid_index = |x, y| x + y * grid_width;
    let mut stone_counter = 0u8;
    let mut stones = vec![];

    let mut algorithm_step = 0;
    let mut grid_image_updater: Option<Object2Updater> = None;
    for x in 0..grid_width {
        for y in 0..grid_height {
            if grid[grid_index(x, y)] != 0 {
                continue;
            }

            let max_x = (grid_width - x).min(parameters.max_stone_width);
            let max_y = {
                let mut max_y = 1;
                for y in (y + 1)..grid_height {
                    if grid[grid_index(x, y)] != 0 {
                        break;
                    }
                    max_y += 1;
                }
                max_y
            }
            .min(parameters.max_stone_height);

            let stone_width = rng.gen_range(1..=max_x);
            let stone_height = rng.gen_range(1..=max_y);
            stone_counter += 1;
            for x in x..(x + stone_width) {
                for y in y..(y + stone_height) {
                    grid[grid_index(x, y)] = stone_counter;
                }
            }
            stones.push(Stone {
                id: stone_counter,
                grid_position: Point2::new(x, y),
                grid_size: Vector2::new(stone_width, stone_height),
            });

            let mut image = Image2Builder::from_grayscale_pixels(
                grid.as_slice(),
                grid_width as u32,
                grid_height as u32,
            )?;
            image.set_per_pixel_transform(PerPixelTransformBuilder::random_distinct_color());
            let mut object2: Object2Builder = image.into();
            object2.add_transform(&Transform2Builder::scale(Vector2Builder::new(
                width, height,
            )));
            object2.set_series_point(&SeriesPointBuilder::new(
                &algorithm_series_id,
                &algorithm_step_dimension_id,
                algorithm_step as f64,
            )?);
            match grid_image_updater.as_ref() {
                None => {
                    grid_image_updater = Some(
                        wall_2d_uploader
                            .upload_object2("grid_image", object2)
                            .await?,
                    );
                }
                Some(updater) => {
                    wall_2d_uploader.update_object2(&updater, object2).await?;
                }
            };
            algorithm_step += 1;
        }
    }

    Ok(stones)
}
