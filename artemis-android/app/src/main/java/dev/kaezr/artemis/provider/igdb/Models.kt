package dev.kaezr.artemis.provider.igdb

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import java.time.Instant

@Serializable
internal data class Game(
    val id: Long,
    val name: String,
    val summary: String? = null,
    val storyline: String? = null,
    val genres: List<Named>,

    @SerialName("first_release_date")
    val firstReleaseDate: Long? = null,
    val cover: Image? = null,
    val artworks: List<Image> = emptyList(),

    @SerialName("involved_companies")
    val involvedCompanies: List<InvolvedCompany>
) {
    @Serializable
    internal data class Named(val name: String)

    @Serializable
    internal data class Image(
        @SerialName("image_id")
        val imageId: String
    )

    @Serializable
    internal data class InvolvedCompany(
        val company: Named,
        val developer: Boolean,
    )
}

internal data class Token(
    val accessToken: String,
    val expiresIn: Instant
)

@Serializable
internal data class TokenResponse(
    @SerialName("access_token")
    val accessToken: String,
    @SerialName("expires_in")
    val expiresIn: Long
) {
    internal fun toToken(): Token {
        return Token(
            accessToken = accessToken,
            expiresIn = Instant.now().plusSeconds(expiresIn)
        )
    }
}