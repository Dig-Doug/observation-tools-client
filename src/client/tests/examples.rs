use observation_tools::artifacts::Image2;
use observation_tools::artifacts::Mesh;
use observation_tools::artifacts::Object2;
use observation_tools::artifacts::PerPixelTransform;
use observation_tools::artifacts::Point2;
use observation_tools::artifacts::Polygon2;
use observation_tools::artifacts::PolygonEdge2;
use observation_tools::artifacts::Rect2;
use observation_tools::artifacts::Segment2;
use observation_tools::artifacts::Text;
use observation_tools::artifacts::Transform2;
use observation_tools::artifacts::Vertex;
use observation_tools::groups::RunUploader;
use observation_tools::Client;
use observation_tools::ClientError;
use observation_tools::ClientOptions;
use observation_tools_common::artifacts::Transform3;

#[test]
fn run_examples() -> Result<(), ClientError> {
    tracing_subscriber::fmt::init();

    let project_id = std::env::var("OBS_TOOLS_PROJECT_ID")
        .expect("Please set the environment variable OBS_TOOLS_PROJECT_ID");
    let api_host = std::env::var("OBS_TOOLS_API_HOST").ok();
    let client = Client::new(
        project_id,
        ClientOptions {
            api_host,
            ..ClientOptions::default()
        },
    )?;

    let run = client.create_run("examples")?;

    let text: Text = "Hello, world!".to_string().into();
    run.create_object1("test", text)?;

    point2_example(&run)?;
    return Ok(());
    image2_example(&run)?;
    mesh3_example(&run)?;
    polygon2_example(&run)?;
    rect2_example(&run)?;
    segment2_example(&run)?;

    println!("See the output at: {}", run.viewer_url());

    Ok(())
}

#[docify::export]
fn point2_example(run: &RunUploader) -> Result<(), ClientError> {
    // Set up a 2D group:
    let group2d = run.child_uploader_2d("point2_world")?;

    // Basic usage:
    group2d.create_object2("my_point", Point2::new(1.0, 2.0))?;
    // Point2s can be created directly from tuples:
    let tuple_point: Point2 = (3.0, 4.0).into();
    group2d.create_object2("my_tuple_point", tuple_point)?;

    // Convert from a nalgebra point:
    let nalgebra_point: Point2 = nalgebra::Point2::new(5.0, 3.0).into();
    group2d.create_object2("nalgebra_point", nalgebra_point)?;

    Ok(())
}

#[docify::export]
fn image2_example(run: &RunUploader) -> Result<(), ClientError> {
    // Set up a 2D group:
    let group2d = run.child_uploader_2d("image2_world")?;

    // Basic usage:
    group2d.create_object2(
        "my_image",
        Image2::new(include_bytes!("../testdata/logo_dark.png"), "image/png"),
    )?;

    // Single-channel images:
    let width = 64;
    let height = 64;
    let mut image = vec![0u8; width * height];
    for y in 0..height {
        for x in 0..width {
            // Make a diagonal pattern
            image[y * width + x] = ((x + y) / 8) as u8;
        }
    }
    let mut single_channel_image =
        Image2::from_grayscale_pixels(&image, width as u32, height as u32)?;
    // Optionally set a per-pixel transform to colorize the image:
    single_channel_image.set_per_pixel_transform(PerPixelTransform::random_distinct_color());
    group2d.create_object2("single_channel_image", single_channel_image)?;

    Ok(())
}

#[docify::export]
fn mesh3_example(run: &RunUploader) -> Result<(), ClientError> {
    // Set up a 3D group:
    let group3d = run.child_uploader_3d("mesh3_world", Transform3::identity())?;

    // Basic usage:
    let mut mesh = Mesh::new();
    mesh.add_vertex(Vertex::new((0.0, 0.0, 0.0)));
    mesh.add_vertex(Vertex::new((1.0, 0.0, 0.0)));
    mesh.add_vertex(Vertex::new((0.0, 1.0, 0.0)));
    mesh.add_triangle(0, 1, 2);
    group3d.create_object3("my_mesh", mesh)?;

    Ok(())
}

#[docify::export]
fn polygon2_example(run: &RunUploader) -> Result<(), ClientError> {
    // Set up a 2D group:
    let group2d = run.child_uploader_2d("polygon2_world")?;

    // Basic usage:
    let mut polygon = Polygon2::new();
    polygon.add_edge(PolygonEdge2::new((0.0, 0.0)));
    polygon.add_edge(PolygonEdge2::new((1.0, 0.0)));
    polygon.add_edge(PolygonEdge2::new((0.0, 1.0)));
    group2d.create_object2("my_polygon", polygon)?;

    // Polygon from list of points:
    let points = vec![
        nalgebra::Point2::new(2.0, 2.0),
        nalgebra::Point2::new(3.0, 2.0),
        nalgebra::Point2::new(3.0, 3.0),
        nalgebra::Point2::new(2.0, 3.0),
    ];
    group2d.create_object2("my_polygon_from_points", Polygon2::from_points(points))?;

    Ok(())
}

#[docify::export]
fn rect2_example(run: &RunUploader) -> Result<(), ClientError> {
    // Set up a 2D group:
    let group2d = run.child_uploader_2d("rect2_world")?;

    // Basic usage:
    group2d.create_object2("my_rect", Rect2::from((10.0, 5.0)))?;

    // Translate the rect, use shorthand (a,b) notation to create vectors and points
    let mut rect2: Object2 = Rect2::from((5.0, 2.5)).into();
    rect2.add_transform(Transform2::translation((2.5, 5.0)));
    group2d.create_object2("translated_rect", rect2)?;

    Ok(())
}

#[docify::export]
fn segment2_example(run: &RunUploader) -> Result<(), ClientError> {
    // Set up a 2D group:
    let group2d = run.child_uploader_2d("segment2_world")?;

    // Basic usage:
    group2d.create_object2("my_segment", Segment2::new((2.0, 1.0), (4.0, 2.0)))?;

    Ok(())
}
