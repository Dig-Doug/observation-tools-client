from observation_tools_client import Client, UserMetadataBuilder, Image2Builder
import cv2
import os
import urllib.request
import numpy as np

TEST_IMAGE_URL = "https://upload.wikimedia.org/wikipedia/en/7/7d/Lenna_%28test_image%29.png"

def main():
    project_id = os.getenv('OBS_PROJECT_ID')
    client = Client(project_id)

    # Create a new run
    run_uploader = client.create_run_blocking()
    print("View your data at: {}".format(run_uploader.viewer_url()))

    run_stage = run_uploader.create_initial_run_stage(UserMetadataBuilder("Run stage"))

    generic_group = run_stage.child_uploader(UserMetadataBuilder("Generic group"))

    with urllib.request.urlopen(TEST_IMAGE_URL) as req:
        image_data = np.asarray(bytearray(req.read()), dtype=np.uint8)
        img = cv2.imdecode(image_data, -1)

        success, encoded_image = cv2.imencode('.png', img)
        if not success:
            raise ValueError('Error encoding image')
        img_builder = Image2Builder(encoded_image.tobytes())

        generic_group.upload_image2(UserMetadataBuilder("Lenna"), img_builder)

if __name__ == "__main__":
    main()
