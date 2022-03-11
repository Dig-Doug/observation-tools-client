from observation_tools_client import Client, UserMetadataBuilder

client = Client("98xvcyvbDcjeZH9nhX6bxz9xoF3")

run_uploader = client.create_run_blocking()

print(run_uploader.viewer_url())

run_stage = run_uploader.create_initial_run_stage(UserMetadataBuilder("Run stage"))

generic_group = run_stage.child_uploader(UserMetadataBuilder("Generic group"))

group_2d = run_stage.child_uploader(UserMetadataBuilder("2D group"))
