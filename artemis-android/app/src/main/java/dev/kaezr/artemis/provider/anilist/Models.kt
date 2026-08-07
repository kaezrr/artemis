package dev.kaezr.artemis.provider.anilist

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
internal data class Response(val data: Data) {
    @Serializable
    internal data class Data(@SerialName("Page") val page: Page)

    @Serializable
    internal data class Page(val media: List<Media>)

    @Serializable
    internal data class Media(
        val id: Long,
        val episodes: UInt? = null,
        val genres: List<String> = emptyList(),
        val coverImage: CoverImage,
        val studios: Studios,
        val title: Title,
        val seasonYear: UInt? = null,
        val description: String? = null,
        val bannerImage: String? = null
    ) {
        @Serializable
        internal data class CoverImage(val extraLarge: String? = null)

        @Serializable
        internal data class Studios(val nodes: List<Studio>) {
            @Serializable
            internal data class Studio(val name: String)
        }

        @Serializable
        internal data class Title(
            val english: String? = null,
            val romaji: String,
        )
    }
}

@Serializable
internal data class Request(
    val query: String,
    val variables: Variables
) {
    @Serializable
    internal data class Variables(
        val search:
        String, val perPage: Int
    )
}