package tools.observation.client

import tools.observation.proto.ArtifactUserMetadata
import tools.observation.proto.ArtifactGroupUploaderData

class RunStageUploader constructor(val client: ObservationToolsClient,
                                   val data: ArtifactGroupUploaderData
) {
    companion object {
        @OptIn(ExperimentalStdlibApi::class)
        fun join(name: String, firstStage: RunStageUploader,
                 vararg otherStages: RunStageUploader): RunStageUploader {
            return firstStage.client.createRunStage(userMetadataFromName(name),
                                                    firstStage.data.projectId,
                                                    firstStage.data.runId,
                                                    firstStage.data.ancestorGroupIdsList,
                                                    buildList {
                                                        add(firstStage.data.id)
                                                        addAll(otherStages.map { it.data.id })
                                                    })
        }
    }

    fun appendStage(name: String): RunStageUploader {
        return client.createRunStage(userMetadataFromName(name),
                                     data.projectId,
                                     data.runId,
                                     data.ancestorGroupIdsList,
                                     listOf(data.id))
    }

    fun childStage(name: String): RunStageUploader {
        return childStage(userMetadataFromName(name))
    }

    @OptIn(ExperimentalStdlibApi::class)
    fun childStage(metadata: ArtifactUserMetadata): RunStageUploader {
        return client.createRunStage(metadata,
                                     data.projectId,
                                     data.runId,
                                     buildList {
                                         addAll(data.ancestorGroupIdsList)
                                         add(data.id)
                                     },
                                     listOf())
    }

    fun serialize(): String {
        return data.toBase64()
    }
}

fun userMetadataFromName(name: String): ArtifactUserMetadata {
    return ArtifactUserMetadata.newBuilder().setName(name).build()
}