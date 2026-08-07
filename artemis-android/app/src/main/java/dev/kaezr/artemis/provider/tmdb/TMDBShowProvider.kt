package dev.kaezr.artemis.provider.tmdb

import dev.kaezr.artemis.provider.ApiProvider
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.accept
import io.ktor.client.request.bearerAuth
import io.ktor.client.request.get
import io.ktor.client.request.parameter
import io.ktor.http.ContentType
import io.ktor.http.Url
import io.ktor.http.appendPathSegments
import io.ktor.http.takeFrom
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import uniffi.artemis.Media
import uniffi.artemis.MediaKind
import uniffi.artemis.ProviderMetadata
import uniffi.artemis.SearchResult

class TMDBShowProvider(
    private val client: HttpClient, private val apiKey: String
) : ApiProvider {
    private val baseUrl = Url("https://api.themoviedb.org/3/")
    override val name = "TMDB"
    override val kind = MediaKind.TV_SHOW

    override suspend fun search(query: String): List<SearchResult> {
        val response: Response<TVShow> = client.get {
            url.takeFrom(baseUrl)
            url.appendPathSegments("search", "tv")

            parameter("query", query)
            parameter("include_adult", true)
            parameter("language", "en-US")
            parameter("page", 1)

            bearerAuth(apiKey)
            accept(ContentType.Application.Json)
        }.body()

        val shows = response.results.take(5)
        val details = fetchShowDetails(shows)

        return shows.map { it.toSearchResult(details.getValue(it.id)) }
    }

    private suspend fun fetchShowDetails(shows: List<TVShow>): Map<Long, TVShowDetailsResponse> =
        coroutineScope {
            shows.map { show ->
                async {
                    val details: TVShowDetailsResponse = client.get {
                        url.takeFrom(baseUrl)
                        url.appendPathSegments("tv", show.id.toString())

                        parameter("append_to_response", "credits")
                        parameter("language", "en-US")

                        bearerAuth(apiKey)
                        accept(ContentType.Application.Json)
                    }.body()

                    show.id to details
                }
            }.awaitAll().toMap()
        }

    private fun TVShow.toSearchResult(details: TVShowDetailsResponse): SearchResult {
        val media = Media.TvShow(
            creator = details.createdBy.firstOrNull()?.name,
            episodes = details.numberOfEpisodes
        )

        val metadata = ProviderMetadata(
            provider = this@TMDBShowProvider.name,
            providerId = id,
            title = this.name,
            coverUrl = posterPath?.let { imageUrl(it, "w500") },
            wideUrl = backdropPath?.let { imageUrl(it, "w1280") },
            description = overview,
            tags = details.genres.map { it.name },
            releaseYear = firstAirDate.take(4).toUIntOrNull(),
        )

        return SearchResult(
            media = media, metadata = metadata, inLibrary = false
        )
    }

    private fun imageUrl(path: String, size: String) = "https://image.tmdb.org/t/p/$size$path"
}
