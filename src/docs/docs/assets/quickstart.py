from observation_tools_client import Client, UserMetadataBuilder
import cv2

def main():
    client = Client("98xvcyvbDcjeZH9nhX6bxz9xoF3")

    # Create a new run
    run_uploader = client.create_run_blocking()
    print("View your data at: {}".format(run_uploader.viewer_url()))

    run_stage = run_uploader.create_initial_run_stage(UserMetadataBuilder("Run stage"))

    generic_group = run_stage.child_uploader(UserMetadataBuilder("Generic group"))

    #img = cv2.imread('g4g.png')
    #encoded_img = cv2.imencode('.png', img)[1]

if __name__ == "__main__":
    main()