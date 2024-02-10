use anyhow::anyhow;
use nalgebra::Isometry3;
use nalgebra::Matrix3;
use nalgebra::Point2;
use nalgebra::Point3;
use nalgebra::Rotation3;
use nalgebra::Transform3;
use nalgebra::Vector2;
use nalgebra::Vector3;
use observation_tools_client::artifacts::Image2Builder;
use observation_tools_client::artifacts::Object2Builder;
use observation_tools_client::artifacts::Object2Updater;
use observation_tools_client::artifacts::PerPixelTransformBuilder;
use observation_tools_client::artifacts::Point2Builder;
use observation_tools_client::artifacts::Polygon2Builder;
use observation_tools_client::artifacts::Polygon3Builder;
use observation_tools_client::artifacts::Rect2Builder;
use observation_tools_client::artifacts::Segment2Builder;
use observation_tools_client::artifacts::SeriesBuilder;
use observation_tools_client::artifacts::SeriesPointBuilder;
use observation_tools_client::artifacts::Transform2Builder;
use observation_tools_client::artifacts::Transform3Builder;
use observation_tools_client::artifacts::Vector2Builder;
use observation_tools_client::groups::ArtifactUploader2d;
use observation_tools_client::groups::ArtifactUploader3d;
use observation_tools_client::ClientOptions;
use observation_tools_client::TokenGenerator;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use tracing::info;
use wasm_bindgen::prelude::*;

pub async fn run_examples(
    project_id: String,
    device_code_auth: bool,
    ui_host: Option<String>,
    api_host: Option<String>,
) -> Result<(), anyhow::Error> {
    let client = observation_tools_client::Client::new(ClientOptions {
        ui_host,
        api_host,
        project_id,
        reqwest_client: None,
        token_generator: if device_code_auth {
            TokenGenerator::OAuth2DeviceCodeFlow
        } else {
            TokenGenerator::OAuth2BrowserFlow
        },
    })?;

    let run_uploader = client.create_run("examples")?;

    let uploader = run_uploader.child_uploader("generic")?;

    let uploader_2d = uploader.child_uploader_2d("upload_basic_example")?;
    upload_basic_example(&uploader_2d)?;

    // TODO(doug): Should we simplify this to just uploader.child_uploader_3d?
    let uploader_3d =
        uploader.child_uploader_3d("generate_barn_wall", Transform3Builder::identity())?;
    generate_stone_wall(&uploader_3d)?;

    println!("See the output at: {}", run_uploader.viewer_url());

    client.shutdown().await?;

    Ok(())
}

#[wasm_bindgen]
pub async fn run_examples_js(
    project_id: String,
    ui_host: Option<String>,
    api_host: Option<String>,
) -> Result<(), JsValue> {
    run_examples(project_id, false, ui_host, api_host)
        .await
        .map_err(|e| JsValue::from_str(&format!("{:?}", e)))
}

fn upload_basic_example(uploader: &ArtifactUploader2d) -> Result<(), anyhow::Error> {
    uploader.create_object2(
        "dinosaur",
        Image2Builder::new(include_bytes!("docusaurus.png"), "image/png"),
    )?;
    uploader.create_object2("point2", Point2Builder::new(1.0, 1.0))?;
    uploader.create_object2(
        "segment2",
        Segment2Builder::new(Point2Builder::new(-1.0, 1.0), Point2Builder::new(1.0, -1.0)),
    )?;
    uploader.create_object2("rect2", Rect2Builder::from(Vector2Builder::new(1.0, 2.0)))?;
    Ok(())
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
        desired_grid_cell_size: 100.0,
        max_stone_grid_width: 1,
        max_stone_grd_height: 1,
        max_stone_rotation_delta_degrees: 2.0,
        max_stone_shrink_percentage: 0.2,
    };

    uploader_3d.create_object3(
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

    wall_2d_uploader.create_object2(
        "wall_profile_local_space",
        Polygon2Builder::from_points(wall_profile_2d.clone()),
    )?;

    let stones = generate_stone_locations(&parameters, &wall_profile_2d, &wall_2d_uploader)?;

    let stones_uploader = wall_2d_uploader.child_uploader_2d("stones")?;
    let mut rng = ChaCha8Rng::seed_from_u64(2);
    for stone in stones {
        let rotation_radians = rng
            .gen_range(
                -parameters.max_stone_rotation_delta_degrees
                    ..=parameters.max_stone_rotation_delta_degrees,
            )
            .to_radians();

        let max_side_length = stone.world_size / (rotation_radians.sin() + rotation_radians.cos());
        let size_variation = rng.gen_range(0.0..=parameters.max_stone_shrink_percentage);
        let final_size = max_side_length * (1.0 - size_variation);

        let mut object2: Object2Builder = Rect2Builder::from(final_size).into();
        let transform = Transform2Builder::from_trs(
            stone.world_position,
            rotation_radians,
            Vector2::from_element(1.0),
        );
        object2.add_transform(transform.clone());
        stones_uploader.create_object2(format!("stone_{}", stone.id), object2)?;

        let mut center2: Object2Builder = Point2Builder::origin().into();
        center2.add_transform(transform);
        stones_uploader.create_object2(format!("stone_center_{}", stone.id), center2)?;
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

    let (bb_min, bb_max) = calculate_bounding_box(&wall_profile_2d)?;
    let size = { bb_max - bb_min };

    let mut rng = ChaCha8Rng::seed_from_u64(2);
    let calculate_num_cells_for_dimension =
        |dimension_size: f64| (dimension_size / parameters.desired_grid_cell_size).floor() as usize;
    let grid_width = calculate_num_cells_for_dimension(size.x);
    let grid_height = calculate_num_cells_for_dimension(size.y);
    let cell_size = size.component_div(&Vector2::new(grid_width as f64, grid_height as f64));
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
                world_position: (bb_min
                    + (cell_size / 2.0)
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
            object2.add_transform(Transform2Builder::from_trs(
                bb_min,
                0.0,
                Vector2Builder::new(size.x, size.y),
            ));
            object2.set_series_point(&SeriesPointBuilder::new(
                &algorithm_series_id,
                &algorithm_step_dimension_id,
                algorithm_step as f64,
            )?);
            match grid_image_updater.as_ref() {
                None => {
                    grid_image_updater = Some(
                        wall_2d_uploader
                            .create_object2("grid_image", object2)?
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
