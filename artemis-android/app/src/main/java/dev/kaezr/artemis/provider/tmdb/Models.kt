package dev.kaezr.artemis.provider.tmdb

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
internal data class Response<T>(val results: List<T>)

@Serializable
internal data class Genre(val name: String)

@Serializable
internal data class Movie(
    val id: Long,

    @SerialName("backdrop_path")
    val backdropPath: String? = null,
    val overview: String? = null,

    @SerialName("poster_path")
    val posterPath: String? = null,

    @SerialName("release_date")
    val releaseDate: String,
    val title: String,
)

@Serializable
internal data class MovieDetailsResponse(
    val runtime: Long? = null,
    val genres: List<Genre>,
    val credits: Credits,
) {
    @Serializable
    internal data class Credits(val crew: List<CrewMember>) {
        @Serializable
        internal data class CrewMember(
            val job: String,
            val name: String
        )
    }
}

@Serializable
internal data class TVShow(
    val id: Long,

    @SerialName("backdrop_path")
    val backdropPath: String? = null,
    val overview: String? = null,

    @SerialName("poster_path")
    val posterPath: String? = null,

    @SerialName("first_air_date")
    val firstAirDate: String,
    val name: String,
)

@Serializable
internal data class TVShowDetailsResponse(
    @SerialName("number_of_episodes")
    val numberOfEpisodes: UInt? = null,

    val genres: List<Genre>,

    @SerialName("created_by")
    val createdBy: List<Person>,
) {
    @Serializable
    internal data class Person(val name: String)
}