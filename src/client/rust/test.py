from observation_tools import Client, UserMetadataBuilder

client = Client("eyJhbGciOiJSUzI1NiIsImtpZCI6IjNkZDZjYTJhODFkYzJmZWE4YzM2NDI0MzFlN2UyOTZkMmQ3NWI0NDYiLCJ0eXAiOiJKV1QifQ.eyJhdWQiOiJodHRwczovL2FwaS5vYnNlcnZhdGlvbi50b29scyIsImF6cCI6IjExNTk1MTQ4NDE3NTEzMzUxNTU2NiIsImVtYWlsIjoiZG91Zy0yNTZAZ3JpZC0yNDkxMTQuaWFtLmdzZXJ2aWNlYWNjb3VudC5jb20iLCJlbWFpbF92ZXJpZmllZCI6dHJ1ZSwiZXhwIjoxNjQ2NDk1OTQ4LCJpYXQiOjE2NDY0OTIzNDgsImlzcyI6Imh0dHBzOi8vYWNjb3VudHMuZ29vZ2xlLmNvbSIsInN1YiI6IjExNTk1MTQ4NDE3NTEzMzUxNTU2NiJ9.TZIRyaVAtNBCmKkEh__-myl-2xhXjdnxUssGKCTUfhdN855CV9YNP2S3czChozhUBy6qGdmg_Goh-0F17bMzRDBiGwWEQSOAOWEd6dc5WhEuzhmy1hVywSfYxU30CJlE1fMbYDUhsrA50mp-lVncTBfX3WOdjRO-224M6o6rk8w3sxaynYmJJlQXVROfSD23PiZLke-i-UdP_H8pNdSBspk7wVeLh8oy75VC-3G5MIp6fYV-6YbS-e04Kzwf_jWN7XPmsluc-oFuyRgB4O6gKB67zFHFRFiwHSEGq9kBF9SVYfW3QnPG6zK01hB1ERKywkixzPBn_SJUpN8Hob_WOw", "98xvdKL41ZYSXGTpvDYSyxQa3m4")

run_uploader = client.create_run()

print(run_uploader.viewer_url())

run_stage = run_uploader.create_initial_run_stage(UserMetadataBuilder("Run stage"))

generic_group = run_stage.child_uploader(UserMetadataBuilder("Generic group"))

group_2d = run_stage.child_uploader(UserMetadataBuilder("2D group"))
