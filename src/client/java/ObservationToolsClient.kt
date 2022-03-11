package tools.observation.client

import io.ktor.client.*
import io.ktor.client.request.forms.*
import io.ktor.http.*
import kotlinx.coroutines.runBlocking
import tools.observation.proto.ArtifactData
import tools.observation.proto.ArtifactGroupUploaderData
import tools.observation.proto.ArtifactId
import tools.observation.proto.ArtifactType
import tools.observation.proto.ArtifactUserMetadata
import tools.observation.proto.CreateArtifactRequest
import tools.observation.proto.CreateRunRequest
import tools.observation.proto.CreateRunResponse
import tools.observation.proto.RunData
import tools.observation.proto.RunId
import tools.observation.proto.RunStageData
import java.time.Instant
import java.util.*

class ObservationToolsClient constructor(val projectId: String,
                                         val httpClient: HttpClient) {
    val host = System.getenv("OBS_HOST").orEmpty()
            .ifBlank { "https://api.observation.tools" }

    suspend fun createRun(): RunUploader {
        val request = CreateRunRequest.newBuilder()
                .setProjectId(projectId)
                .setRunData(RunData.newBuilder()
                                    .setClientCreationTime(Instant.now().toProto()))
                .build()

        val response = httpClient.submitForm<String>(
                url = "${host}/create-run",
                formParameters = Parameters.build {
                    append("request", request.toBase64())
                },
                encodeInQuery = false
        )

        val createRunResponse = response.base64ToProto(CreateRunResponse.getDefaultInstance())
        return RunUploader(this, createRunResponse)
    }

    fun deserializeRunStage(serialized: String): RunStageUploader {
        return RunStageUploader(this,
                                serialized.base64ToProto(
                                        ArtifactGroupUploaderData.getDefaultInstance()))
    }

    @OptIn(ExperimentalStdlibApi::class)
    fun createRunStage(userMetadata: ArtifactUserMetadata,
                       projectId: String,
                       runId: RunId,
                       ancestorGroupIds: List<ArtifactId>,
                       previousStageIds: List<ArtifactId>): RunStageUploader {
        // TODO(doug): Check all inputs have the same parent
        val request = CreateArtifactRequest.newBuilder()
                .setProjectId(projectId)
                .setRunId(runId)
                .setArtifactId(ArtifactId.newBuilder()
                                       .setUuid(UUID.randomUUID().toProto())
                                       .build()
                )
                .setArtifactData(ArtifactData.newBuilder()
                                         .setUserMetadata(userMetadata)
                                         .setArtifactType(ArtifactType.ARTIFACT_TYPE_RUN_STAGE)
                                         .setRunStageData(RunStageData.newBuilder()
                                                                  .addAllPreviousRunStageIds(
                                                                          previousStageIds)
                                         )
                                         .addAllAncestorGroupIds(ancestorGroupIds)
                                         .setClientCreationTime(Instant.now().toProto()))
                .build()

        runBlocking {
            httpClient.submitFormWithBinaryData<String>(
                    url = "${host}/create-artifact",
                    formData = formData {
                        append("request", request.toBase64())
                    }
            )
        }

        return RunStageUploader(this, ArtifactGroupUploaderData.newBuilder()
                .setProjectId(projectId)
                .setRunId(runId)
                .setId(request.artifactId)
                .addAllAncestorGroupIds(ancestorGroupIds)
                .build())
    }
}