package dev.kaezr.artemis.provider.igdb

import dev.kaezr.artemis.provider.ApiProvider
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.bearerAuth
import io.ktor.client.request.header
import io.ktor.client.request.parameter
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.http.Url
import uniffi.artemis.Media
import uniffi.artemis.MediaKind
import uniffi.artemis.ProviderMetadata
import uniffi.artemis.SearchResult
import java.time.Instant
import java.time.ZoneOffset

class IGDBProvider(
    private val client: HttpClient,
    private val clientId: String,
    private val clientSecret: String,
) : ApiProvider {
    private val baseUrl = Url("https://api.igdb.com/v4/games")
    private var token: Token? = null

    override val name = "IGDB"
    override val kind = MediaKind.GAME

    override suspend fun search(query: String): List<SearchResult> {
        val accessToken = getAccessToken()

        val body = """
        search "$query";
        fields
            id,
            name,
            summary,
            storyline,
            genres.name,
            first_release_date,
            cover.image_id,
            artworks.image_id,
            involved_companies.company.name,
            game_type.type,
            involved_companies.developer;
        where game_type = 0 &
            genres != null &
            involved_companies != null;
        limit 5;""".trimIndent()

        val response: List<Game> = client.post(baseUrl) {
            header("Client-ID", clientId)
            bearerAuth(accessToken)
            setBody(body)
        }.body()

        return response.map { it.toSearchResult() }
    }


    private suspend fun getAccessToken(): String {
        val needsRefresh = token == null || Instant.now().plusSeconds(30).isAfter(token!!.expiresIn)

        if (needsRefresh) {
            val response: TokenResponse = client.post("https://id.twitch.tv/oauth2/token") {
                parameter("client_id", clientId)
                parameter("client_secret", clientSecret)
                parameter("grant_type", "client_credentials")
            }.body()

            token = response.toToken()
        }

        return token!!.accessToken
    }

    private fun imageUrl(hash: String, size: String) =
        "https://images.igdb.com/igdb/image/uploads/t_$size/$hash.jpg"

    private fun Game.toSearchResult(): SearchResult {
        val media = Media.Game(
            developer = this.involvedCompanies.firstOrNull { it.developer }?.company?.name,
            playtime = null,
        )

        val metadata = ProviderMetadata(
            provider = this@IGDBProvider.name,
            providerId = id,
            title = this.name,
            coverUrl = cover
                ?.imageId
                ?.let { imageUrl(it, "cover_big") },

            wideUrl = artworks
                .firstOrNull()
                ?.imageId
                ?.let { imageUrl(it, "1080p") },

            description = storyline ?: summary,
            tags = genres.map { it.name },
            releaseYear = firstReleaseDate
                ?.let { Instant.ofEpochSecond(it) }
                ?.atOffset(ZoneOffset.UTC)
                ?.year
                ?.toUInt())

        return SearchResult(
            media = media,
            metadata = metadata,
            inLibrary = false,
        )
    }
}