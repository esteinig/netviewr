#' NetView R
#'
#' Generate mutual nearest-neighbour graphs for analysis of population structure and visualization with iGraph.
#'
#' @param dist        Symmetrical distance matrix for NetView (N x N) [ matrix ]
#' @param k           Range of parameter k for mutual k-nearest-neighbour search [ int vector, 10:60 ]
#' @param mutual      Construct mutual nearest neighbor graph instead of nearest neighbor graph [ bool, TRUE ]
#' @param weights     Weight of edges is mapped to pairwise distance in input matrix [ bool, TRUE ]
#' @param mst         Include edges of the minimum spanning tree associated with the data [ bool, FALSE ]
#' @param algorithm   Algorithm for mutual nearest neighbour search [ char cover_tree ]
#'
#' @return List of graph objects (igraph)
#'
#' @usage netview(dist, k=1:60, mutual=T, weights=T, mst=F, algorithm='cover_tree')
#'
#' @details For examples and tutorials, please see the repository: \url{https://github.com/esteinig/netviewr}
#'
#' @export
#' @import igraph
#' @import cccd

netview <- function(dist=NULL, k=20, mutual=TRUE, weights=TRUE, mst=FALSE, algorithm='cover_tree'){

  # See original source code of SPC algorithm implemented in SPIN - Neuditschko et al. (2012)

  if(is.matrix(dist)==F | nrow(dist) != ncol(dist) | is.null(dist)) {
    stop('Input must be a symmetric distance matrix (N x N).')
  }

  if(!is.vector(k) && is.numeric(k)){
    k <- c(k)
  }

  graphs <- list()
  for (i in k) {
    mknn_graph <- cccd::nng(dist, k=i, mutual=mutual, algorithm=algorithm)
    V(mknn_graph)$name <- seq(length(V(mknn_graph)))
    mknn_graph$layout <- igraph::layout.fruchterman.reingold(mknn_graph)
    mknn_graph$dist <- dist

    if (mst){
      fc_graph <- igraph::graph.adjacency(mdist, mode=c('undirected'), weighted=TRUE)
      mst_graph <- igraph::minimum.spanning.tree(fc_graph)
      V(mst_graph)$name <- seq(length(V(mst_graph)))
      g <- igraph::simplify(mknn_graph + edge(as.vector(t(ends(mst_graph, E(mst_graph))))))
    } else g <- mknn_graph

    if (weights) E(g)$weight <- g$dist[ends(g, E(g))]

    graphs[[paste0('k', as.character(i))]] <- g
  }

  return(graphs)
}

