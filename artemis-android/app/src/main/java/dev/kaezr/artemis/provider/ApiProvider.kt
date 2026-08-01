package dev.kaezr.artemis.provider

import uniffi.artemis.MediaKind
import uniffi.artemis.SearchResult

interface ApiProvider {
    val name: String
    val kind: MediaKind
    suspend fun search(query: String): List<SearchResult>
}