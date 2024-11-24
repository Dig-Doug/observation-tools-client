import logging
import time

from observation_tools import Client, UserMetadata, Text, Object1

FORMAT = '%(levelname)s %(name)s %(asctime)-15s %(filename)s:%(lineno)d %(message)s'
logging.basicConfig(format=FORMAT)
logging.getLogger().setLevel(logging.DEBUG)

logger = logging.getLogger(__name__)

logger.info("Starting example")
client = Client("p_Tgm3GjQSZPkPWkFcjnQBAuYVNY", api_host="http://localhost:8000")

logger.info("Creating run")
run_uploader = client.create_run(UserMetadata("py_example"))

run_uploader.create_object1(UserMetadata("object1"), Object1(Text("Hello, world!")))

logger.info("Shutting down client")
client.shutdown()
