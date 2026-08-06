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
import java.time.Duration

class TMDBMovieProvider(
    private val client: HttpClient, private val apiKey: String
) : ApiProvider {
    private val baseUrl = Url("https://api.themoviedb.org/3/")
    override val name = "TMDB"
    override val kind = MediaKind.MOVIE

    override suspend fun search(query: String): List<SearchResult> {
        val response: Response<Movie> = client.get {
            url.takeFrom(baseUrl)
            url.appendPathSegments("search", "movie")

            parameter("query", query)
            parameter("include_adult", true)
            parameter("language", "en-US")
            parameter("page", 1)

            bearerAuth(apiKey)
            accept(ContentType.Application.Json)
        }.body()

        val movies = response.results.take(5)
        val details = fetchMovieDetails(movies)

        return movies.map { it.toSearchResult(details.getValue(it.id)) }
    }

    private suspend fun fetchMovieDetails(movies: List<Movie>): Map<Long, MovieDetailsResponse> =
        coroutineScope {
            movies.map { movie ->
                async {
                    val details: MovieDetailsResponse = client.get {
                        url.takeFrom(baseUrl)
                        url.appendPathSegments("movie", movie.id.toString())

                        parameter("append_to_response", "credits")
                        parameter("language", "en-US")

                        bearerAuth(apiKey)
                        accept(ContentType.Application.Json)
                    }.body()

                    movie.id to details
                }
            }.awaitAll().toMap()
        }

    private fun Movie.toSearchResult(details: MovieDetailsResponse): SearchResult {
        val media = Media.Movie(
            director = details.credits.crew.firstOrNull { it.job == "Director" }?.name,
            duration = details.runtime?.let(Duration::ofMinutes)
        )

        val metadata = ProviderMetadata(
            provider = name,
            providerId = id,
            title = title,
            coverUrl = posterPath?.let { imageUrl(it, "w500") },
            wideUrl = backdropPath?.let { imageUrl(it, "w1280") },
            description = overview,
            tags = details.genres.map { it.name },
            releaseYear = releaseDate.take(4).toUIntOrNull(),
        )

        return SearchResult(
            media = media, metadata = metadata, inLibrary = false
        )
    }

    private fun imageUrl(path: String, size: String) = "https://image.tmdb.org/t/p/$size$path"
}