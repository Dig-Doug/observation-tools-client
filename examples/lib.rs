use nalgebra::{
    Affine3, Isometry3, Matrix3, Point2, Point3, Quaternion, Rotation3, Transform3, Translation3,
    Vector2, Vector3,
};
use observation_tools_client::builders::{
    Object3Builder, Polygon3Builder, PolygonEdge3Builder, Transform3Builder,
};
use observation_tools_client::{ArtifactUploader3d, GenericArtifactUploader, UserMetadataBuilder};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::error::Error;
use tracing::info;

pub type GenericError = Box<dyn Error + Send + Sync>;

struct Stone {
    id: usize,
    grid_position: Point2<usize>,
    grid_size: Vector2<usize>,
}

// Algorithm for procedurally generating a wall for an old wooden barn ([example image](https://en.wikipedia.org/wiki/Barn#/media/File:MBL_Olsztynek_-_15b._Budynek_gospodarczy_z_Kwietniewa.jpg)).
//
// NOTE:
// - The algorithm assumes that the input wall profile is perpendicular to the ground plane.
pub async fn generate_barn_wall(uploader_3d: ArtifactUploader3d) -> Result<(), GenericError> {
    let wall_profile_world = vec![
        Point3::new(108.0, 64.0, 13.0),
        Point3::new(108.0, 64.0, 15.0),
        Point3::new(111.0, 66.0, 16.0),
        Point3::new(114.0, 68.0, 15.0),
        Point3::new(114.0, 68.0, 13.0),
    ];
    let connection_points = vec![0.0, 0.5, 1.5, 2.0, 2.5, 3.0];
    let desired_plank_width = 0.1;

    uploader_3d
        .upload_object3(
            "wall_profile_world_space",
            Polygon3Builder::from_points(wall_profile_world.clone()),
        )
        .await?;

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
    let world_to_local_transform = local_to_world_transform.inverse();
    // TODO(doug): Remove need to convert to Transform3 manually
    let local_to_world_transform: Transform3<f64> = nalgebra::convert(local_to_world_transform);

    let wall_profile_local: Vec<Point3<f64>> = wall_profile_world
        .iter()
        .map(|p| world_to_local_transform * p)
        .collect();

    let local_space_uploader = uploader_3d
        .child_uploader_3d(
            &UserMetadataBuilder::new("local_space"),
            local_to_world_transform.into(),
        )
        .await?;

    local_space_uploader
        .upload_object3(
            "wall_profile_local_space",
            Polygon3Builder::from_points(wall_profile_local.clone()),
        )
        .await?;

    for p in wall_profile_world.iter() {
        let p_local = world_to_local_transform * p;
        println!("p_local: {:?}", p_local);
    }

    // TODO(doug): Create a grid pattern
    let width = 10.0;
    let plank_column_count = (width / desired_plank_width) as f64;
    let plank_column_count = plank_column_count.floor() as usize;
    let max_stone_width = 3;

    let mut rng = ChaCha8Rng::seed_from_u64(2);
    let mut grid = vec![vec![0; connection_points.len() - 1]; plank_column_count];
    let mut stone_counter = 0;
    let mut stones = vec![];
    for x in 0..plank_column_count {
        for y in 0..grid.len() {
            if grid[x][y] != 0 {
                continue;
            }

            let max_x = (plank_column_count - x).min(max_stone_width);
            let max_y = {
                let mut max_y = 1;
                for y in (y + 1)..grid.len() {
                    if grid[x][y] != 0 {
                        break;
                    }
                    max_y += 1;
                }
                max_y
            };

            let stone_width = rng.gen_range(1..max_x);
            let stone_height = rng.gen_range(1..max_y);
            stone_counter += 1;
            for x in x..(x + stone_width) {
                for y in y..(y + stone_height) {
                    grid[x][y] = stone_counter;
                }
            }
            stones.push(Stone {
                id: stone_counter,
                grid_position: Point2::new(x, y),
                grid_size: Vector2::new(stone_width, stone_height),
            });
        }
    }

    // TODO(doug): Export an image where the pattern will be
    // TODO(doug): Convert the grid pattern to boxes
    // TODO(doug): Apply adjustments to boxes to add randomness
    // TODO(doug): Mask the boxes to the wall profile

    Ok(())
}
