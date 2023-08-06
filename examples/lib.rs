use anyhow::anyhow;
use clap::Parser;
use nalgebra::Affine3;
use nalgebra::Isometry2;
use nalgebra::Isometry3;
use nalgebra::Matrix3;
use nalgebra::Point2;
use nalgebra::Point3;
use nalgebra::Quaternion;
use nalgebra::Rotation3;
use nalgebra::Transform;
use nalgebra::Transform3;
use nalgebra::Translation2;
use nalgebra::Translation3;
use nalgebra::UnitComplex;
use nalgebra::Vector2;
use nalgebra::Vector3;
use observation_tools_client::builders::Image2Builder;
use observation_tools_client::builders::Object2Builder;
use observation_tools_client::builders::Object2Updater;
use observation_tools_client::builders::Object3Builder;
use observation_tools_client::builders::PerPixelTransformBuilder;
use observation_tools_client::builders::Point2Builder;
use observation_tools_client::builders::Polygon2Builder;
use observation_tools_client::builders::Polygon3Builder;
use observation_tools_client::builders::PolygonEdge3Builder;
use observation_tools_client::builders::Rect2Builder;
use observation_tools_client::builders::SeriesBuilder;
use observation_tools_client::builders::SeriesPointBuilder;
use observation_tools_client::builders::SphereBuilder;
use observation_tools_client::builders::Transform2Builder;
use observation_tools_client::builders::Transform3Builder;
use observation_tools_client::builders::UserMetadataBuilder;
use observation_tools_client::builders::Vector2Builder;
use observation_tools_client::uploaders::ArtifactUploader2d;
use observation_tools_client::uploaders::ArtifactUploader3d;
use observation_tools_client::ClientError;
use observation_tools_client::ClientOptions;
use observation_tools_client::TokenGenerator;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use tracing::info;
use wasm_bindgen::prelude::*;

pub async fn run_examples(
    project_id: String,
    auth_token: String,
    ui_host: Option<String>,
    api_host: Option<String>,
) -> Result<(), anyhow::Error> {
    let client = observation_tools_client::Client::new(ClientOptions {
        ui_host,
        api_host,
        project_id,
        client: None,
        token_generator: TokenGenerator::Constant(auth_token),
    })
    .expect("Failed to create client");

    let run_uploader = client.create_run(&UserMetadataBuilder::new("examples"))?;

    let uploader = run_uploader.child_uploader(&UserMetadataBuilder::new("generic"))?;
    // TODO(doug): Should we simplify this to just uploader.child_uploader_3d?
    let uploader_3d = uploader.child_uploader_3d(
        &UserMetadataBuilder::new("generate_barn_wall"),
        Transform3Builder::identity(),
    )?;
    generate_stone_wall(&uploader_3d)?;

    info!("See the output at: {}", run_uploader.viewer_url());

    client.shutdown().await?;

    Ok(())
}

#[wasm_bindgen]
pub async fn run_examples_js(
    project_id: String,
    auth_token: String,
    ui_host: Option<String>,
    api_host: Option<String>,
) -> Result<(), JsValue> {
    run_examples(project_id, auth_token, ui_host, api_host)
        .await
        .map_err(|e| JsValue::from_str(&format!("{:?}", e)))
}

struct AlgorithmParameters {
    desired_grid_cell_size: f64,
    max_stone_grid_width: usize,
    max_stone_grd_height: usize,
    max_stone_rotation_delta_degrees: f64,
    max_stone_shrink_percentage: f64,
}

// Algorithm for procedurally generating a stone wall.
pub fn generate_stone_wall(uploader_3d: &ArtifactUploader3d) -> Result<(), anyhow::Error> {
    let wall_profile_world = vec![
        Point3::new(10800.0, 6400.0, 1300.0),
        Point3::new(10800.0, 6400.0, 1500.0),
        Point3::new(11400.0, 6800.0, 1500.0),
        Point3::new(11400.0, 6800.0, 1300.0),
    ];
    let parameters = AlgorithmParameters {
        desired_grid_cell_size: 50.0,
        max_stone_grid_width: 1,
        max_stone_grd_height: 1,
        max_stone_rotation_delta_degrees: 2.0,
        max_stone_shrink_percentage: 0.02,
    };

    uploader_3d.upload_object3(
        "wall_profile_world_space",
        Polygon3Builder::from_points(wall_profile_world.clone()),
    )?;

    let (world_to_local_transform, local_to_world_transform) =
        calculate_world_to_local_space_transforms(&wall_profile_world)?;

    let wall_profile_2d: Vec<Point2<f64>> = wall_profile_world
        .iter()
        .map(|p| world_to_local_transform * p)
        .map(|p| Point2::new(p.x, p.y))
        .collect();

    let wall_2d_uploader = uploader_3d.child_uploader_2d("wall_2d", local_to_world_transform)?;

    wall_2d_uploader.upload_object2(
        "wall_profile_local_space",
        Polygon2Builder::from_points(wall_profile_2d.clone()),
    )?;

    let stones = generate_stone_locations(&parameters, &wall_profile_2d, &wall_2d_uploader)?;

    let stones_uploader = wall_2d_uploader.child_uploader_2d("stones")?;
    let mut rng = ChaCha8Rng::seed_from_u64(2);
    for stone in stones {
        let size_variation = rng.gen_range(0.0..=parameters.max_stone_shrink_percentage);
        let final_size = stone.world_size * (1.0 - size_variation);

        let rotation_degrees = rng.gen_range(
            -parameters.max_stone_rotation_delta_degrees
                ..=parameters.max_stone_rotation_delta_degrees,
        );

        Isometry2::from_parts(
            Translation2::from(stone.world_position),
            UnitComplex::from_angle(rotation_degrees.to_radians()),
        );

        let mut object2: Object2Builder = Rect2Builder::from(final_size).into();
        object2.add_transform(&Transform2Builder::from_trs(
            stone.world_position,
            rotation_degrees.to_radians(),
            Vector2::from_element(1.0),
        ));
        stones_uploader.upload_object2(format!("stone_{}", stone.id), object2)?;

        let center: Point2Builder = stone.world_position.clone().into();
        let mut center2: Object2Builder = (&center).into();
        center2.add_transform(&Transform2Builder::from_trs(
            stone.world_position,
            0.0,
            Vector2::from_element(1.0),
        ));
        stones_uploader.upload_object2(format!("stone_center_{}", stone.id), center2)?;
    }

    // TODO(doug): Convert the grid pattern to boxes

    Ok(())
}

fn calculate_world_to_local_space_transforms(
    wall_profile_world: &Vec<Point3<f64>>,
) -> Result<(Transform3<f64>, Transform3<f64>), anyhow::Error> {
    // Calculate the normal of the wall
    let side1 = wall_profile_world[2] - wall_profile_world[1];
    let side2 = wall_profile_world[0] - wall_profile_world[1];
    let global_z = side1.cross(&side2).normalize();
    let global_y = Vector3::z();
    let global_x = global_y.cross(&global_z).normalize();

    // Define a local coordinate system that aligns the face on the XY plane
    let origin = wall_profile_world[0];
    let local_to_world_transform = Isometry3::from_parts(
        origin.into(),
        Rotation3::from_matrix_unchecked(Matrix3::from_columns(&[global_x, global_y, global_z]))
            .into(),
    );
    let world_to_local_transform: Transform3<f64> =
        nalgebra::convert(local_to_world_transform.inverse());
    // TODO(doug): Remove need to convert to Transform3 manually
    let local_to_world_transform: Transform3<f64> = nalgebra::convert(local_to_world_transform);

    Ok((world_to_local_transform, local_to_world_transform))
}

struct GridStone {
    id: u8,
    world_position: Point2<f64>,
    world_size: Vector2<f64>,
}

fn generate_stone_locations(
    parameters: &AlgorithmParameters,
    wall_profile_2d: &Vec<Point2<f64>>,
    wall_2d_uploader: &ArtifactUploader2d,
) -> Result<Vec<GridStone>, anyhow::Error> {
    let mut series_builder = SeriesBuilder::new();
    let algorithm_step_dimension_id = series_builder.add_dimension("algorithm_step");
    let algorithm_series_id = wall_2d_uploader.series("grid_algorithm", series_builder)?;

    let size = {
        let (min, max) = calculate_bounding_box(&wall_profile_2d)?;
        max - min
    };

    let mut rng = ChaCha8Rng::seed_from_u64(2);
    let calculate_num_cells_for_dimension =
        |dimension_size: f64| (dimension_size / parameters.desired_grid_cell_size).floor() as usize;
    let grid_width = calculate_num_cells_for_dimension(size.x);
    let grid_height = calculate_num_cells_for_dimension(size.y);
    let cell_size = size / (grid_width as f64);
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

            let max_x = (grid_width - x).min(parameters.max_stone_grid_width);
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
            .min(parameters.max_stone_grd_height);

            let stone_width = rng.gen_range(1..=max_x);
            let stone_height = rng.gen_range(1..=max_y);
            stone_counter += 1;
            for x in x..(x + stone_width) {
                for y in y..(y + stone_height) {
                    grid[grid_index(x, y)] = stone_counter;
                }
            }
            stones.push(GridStone {
                id: stone_counter,
                world_position: (Point2::origin() - (cell_size / 2.0)
                    + Vector2::new(x, y).cast().component_mul(&cell_size)),
                world_size: Vector2::new(stone_width, stone_height)
                    .cast()
                    .component_mul(&cell_size),
            });

            let mut image = Image2Builder::from_grayscale_pixels(
                grid.as_slice(),
                grid_width as u32,
                grid_height as u32,
            )?;
            image.set_per_pixel_transform(PerPixelTransformBuilder::random_distinct_color());
            let mut object2: Object2Builder = image.into();
            object2.add_transform(&Transform2Builder::scale(Vector2Builder::new(
                size.x, size.y,
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
                            .upload_object2("grid_image", object2)?
                            .result,
                    );
                }
                Some(updater) => {
                    wall_2d_uploader.update_object2(&updater, object2)?;
                }
            };
            algorithm_step += 1;
        }
    }

    Ok(stones)
}

fn calculate_bounding_box(
    wall_profile_2d: &Vec<Point2<f64>>,
) -> Result<(Point2<f64>, Point2<f64>), anyhow::Error> {
    let mut min: Point2<f64> = wall_profile_2d
        .first()
        .cloned()
        .ok_or(anyhow!("Wall profile is empty"))?;
    let mut max: Point2<f64> = min.clone();
    for point in wall_profile_2d.iter() {
        min = min.inf(&point);
        max = max.sup(&point);
    }
    Ok((min, max))
}
