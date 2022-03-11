package tools.observation.client

import com.google.protobuf.Message
import com.google.protobuf.Timestamp
import java.time.Duration
import java.time.Instant
import java.util.*

fun Message.toBase64(): String {
    return Base64.getEncoder().encodeToString(toByteArray())
}

fun <T : Message> String.base64ToProto(proto: T): T {
    @Suppress("UNCHECKED_CAST")
    return proto.parserForType.parseFrom(Base64.getDecoder().decode(this)) as T
}

fun Message.toBase64UrlSafe(): String {
    return Base64.getUrlEncoder().encodeToString(toByteArray())
}

fun <T : Message> String.base64UrlSafeToProto(proto: T): T {
    @Suppress("UNCHECKED_CAST")
    return proto.parserForType.parseFrom(Base64.getUrlDecoder().decode(this)) as T

}

fun Timestamp.toInstant(): Instant {
    return Instant.ofEpochSecond(seconds, nanos.toLong())
}

fun Instant.toProto(): Timestamp {
    return Timestamp.newBuilder().setSeconds(epochSecond).setNanos(nano).build()
}

fun com.google.protobuf.Duration.toJava(): Duration {
    return Duration.ofSeconds(seconds)
}

fun Duration.toProto(): com.google.protobuf.Duration {
    return com.google.protobuf.Duration.newBuilder()
            .setSeconds(seconds)
            .setNanos(nano)
            .build()
}
