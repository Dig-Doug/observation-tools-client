package tools.observation.client

import com.google.protobuf.ByteString
import tools.observation.proto.Uuid
import java.nio.ByteBuffer
import java.util.*

fun uuidFromBytes(bytes: ByteArray): UUID {
    assert(bytes.size == 16)
    val buffer = ByteBuffer.wrap(bytes)
    val first = buffer.getLong()
    val second = buffer.getLong()
    return UUID(first, second)
}

fun uuidToBytes(uuid: UUID): ByteArray {
    val buffer = ByteBuffer.wrap(ByteArray(16))
    buffer.putLong(uuid.mostSignificantBits)
    buffer.putLong(uuid.leastSignificantBits)
    return buffer.array()
}

fun uuidToBase64(uuid: UUID): String {
    return Base64.getUrlEncoder().encodeToString(uuidToBytes(uuid))
}

fun uuidFromBase64(base64: String): UUID {
    return uuidFromBytes(Base64.getUrlDecoder().decode(base64))
}

fun Uuid.toJava(): UUID {
    return uuidFromBytes(this.data.toByteArray())
}

fun UUID.toProto(): Uuid {
    return Uuid.newBuilder()
            .setData(ByteString.copyFrom(uuidToBytes(this)))
            .build()
}
