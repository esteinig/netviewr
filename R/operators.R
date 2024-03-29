#' Meta Operator (Column Concatenation Operator)
#'
#' Adds a Tibble to the graph attribute `meta` (or concatenates to existing Tibble)
#' for use in subsequent graph decorators that might use the data argument to select
#' columns from Tibble:
#'
#'@name meta_operator
#'@param t1     data frame or tibble containing some data
#'@param t2     data frame or tibble containing some other data to merge
#'
#'@return Merged data frame by columns of t1 and t2
#'
#'@export
`%m%` = `%c%` = function(t1, t2) {
  dplyr::bind_cols(tibble::as_tibble(t1), tibble::as_tibble(t2))
}

#'Graph Decorator Operator
#'
#'Decorate a graph object (g) a decorator function (f) or a tibble with meta data
#'for nodes (length: nodes)
#'Functions can be defined manually as functions that return an anonymous function.
#'
#'@name graph_decorator
#'@param g     graph object or list of graph objects to decorate
#'@param f     decorator function to decorate the graph with, or a tibble
#'
#'@return Graph object or list of graph objects with decorated attributes
#'
#'@export
`%@%` =`%g%` = function(g, f) {

  # Operator function:
  op <- function(g, f){

    if (!igraph::is.igraph(g)) stop('Left-hand input to graph operator must be a graph object or list of graph objects.')

    if (purrr::is_function(f)) {

      g <- f(g)

    } else if (is.data.frame(f) | tibble::is_tibble(f)){

      if (nrow(f) == igraph::vcount(g)){

        if (is.null(g$node_data)) g$node_data <- tibble::as_tibble(f) else g$node_data <- dplyr::bind_cols(g$node_data, tibble::as_tibble(f))

      } else stop('Tibble input must be of length of the number of nodes')

      # Note here to add support for edges in later version
      # else if (nrow(f) == ecount(g)){
      #if (is_null(g$edge_data)) g$edge_data <- as_tibble(f) else g$edge_data <- dplyr::bind_cols(g$edge_data, as_tibble(f))
      #}

    }

    else stop('Right-hand input to operator must be a decorator function that modifies
               and returns the graph object, or a DataFrame / Tibble.')

    return(g)

  }

  # Input is graph or list of graphs:
  if (igraph::is.igraph(g)) op(g, f) else sapply(g, op, f=f, simplify = FALSE, USE.NAMES = TRUE)

}

#' Magrittr Pipe
#' @importFrom magrittr %>%
#' @export
magrittr::`%>%`
