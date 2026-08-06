package dev.kaezr.artemis.provider.tmdb

import kotlinx.serialization.Serializable

@Serializable
internal data class Response<T>(val results: List<T>)

@Serializable
internal data class Genre(val name: String)

@Serializable
internal data class Movie(
    val id: Long,
    val backdropPath: String?,
    val overview: String?,
    val posterPath: String?,
    val releaseDate: String,
    val title: String,
)

@Serializable
internal data class MovieDetailsResponse(
    val runtime: UInt?,
    val genres: List<Genre>,
    val credits: Credits,
) {
    @Serializable
    internal data class Credits(val crew: List<CrewMember>) {
        @Serializable
        internal data class CrewMember(val job: String, val name: String)
    }
}

@Serializable
internal data class TVShow(
    val id: Long,
    val backdropPath: String?,
    val overview: String?,
    val posterPath: String?,
    val firstAirDate: String,
    val name: String,
)

@Serializable
internal data class TVShowDetailsResponse(
    val numberOfEpisodes: UInt?,
    val genres: List<Genre>,
    val createdBy: List<Person>,
) {
    @Serializable
    internal data class Person(val name: String)
}