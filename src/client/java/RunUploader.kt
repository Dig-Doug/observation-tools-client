package tools.observation.client

import tools.observation.proto.ArtifactGroupUploaderData
import tools.observation.proto.CreateRunResponse

class RunUploader constructor(val client: ObservationToolsClient,
                              val createRunResponse: CreateRunResponse) {
    fun serialize(): String {
        return createRunResponse.toBase64()
    }

    fun createInitialRunStage(name: String): RunStageUploader {
        return client.createRunStage(userMetadataFromName(name),
                                     client.projectId,
                                     createRunResponse.runId,
                                     listOf(createRunResponse.rootStageId),
                                     listOf())
    }
}