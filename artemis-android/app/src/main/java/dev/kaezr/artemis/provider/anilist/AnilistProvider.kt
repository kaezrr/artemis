package dev.kaezr.artemis.provider.anilist

import dev.kaezr.artemis.provider.ApiProvider
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.http.ContentType
import io.ktor.http.contentType
import uniffi.artemis.Media
import uniffi.artemis.MediaKind
import uniffi.artemis.ProviderMetadata
import uniffi.artemis.SearchResult

private const val QUERY = """
query(${'$'}search: String!, ${'$'}perPage: Int) {
  Page(perPage: ${'$'}perPage) {
    media(search: ${'$'}search, type: ANIME, sort: POPULARITY_DESC) {
      id
      episodes
      genres
      coverImage { extraLarge }
      studios(isMain: true) { nodes { name } }
      title { english romaji }
      seasonYear
      description(asHtml: false)
      bannerImage
    }
  }
}
"""

class AnilistProvider(private val client: HttpClient) : ApiProvider {
    override val name = "AniList"
    override val kind = MediaKind.ANIME

    override suspend fun search(query: String): List<SearchResult> {
        val response: Response = client.post("https://graphql.anilist.co") {
            contentType(ContentType.Application.Json)
            setBody(
                Request(
                    query = QUERY,
                    variables = Request.Variables(search = query, perPage = 5)
                )
            )
        }.body()

        return response.data.page.media.map { it.toSearchResult() }
    }

    private fun Response.Media.toSearchResult(): SearchResult {
        val media = Media.Anime(
            studio = studios.nodes.firstOrNull()?.name,
            episodes = episodes
        )

        val metadata = ProviderMetadata(
            provider = name,
            providerId = id,
            title = title.english ?: title.romaji,
            coverUrl = coverImage.extraLarge,
            wideUrl = bannerImage,
            description = description,
            tags = genres,
            releaseYear = seasonYear,
        )

        return SearchResult(
            media,
            metadata,
            inLibrary = false
        )
    }
}