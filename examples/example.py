from observation_tools import Client, UserMetadata, Text, Object1

# Setup client
client = Client("p_Tgm3GjQSZPkPWkFcjnQBAuYVNY", api_host="http://localhost:8000")

run_uploader = client.create_run(UserMetadata("py_example"))

run_uploader.create_object1(UserMetadata("object1"), Object1(Text("Hello, world!")))

run_uploader.create_object1(name="llm_output", data="Hello, world!")


def process1():
    run_uploader.create_object1(name="ocr_output", data="Hello, world!")
    run_uploader.create_object1(name="llm_output", data="Hello, world!")


def process2():
    group = run_uploader.create_group(UserMetadata("process2"))
    group.create_object1(name="ocr_output2", data="Hello, world!")
    group.create_object1(name="llm_output2", data="Hello, world!")


client.shutdown()
