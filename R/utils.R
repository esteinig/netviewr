#' Netview Plot
#'@param g                igraph object: the igraph object to plot
#'@param legend           char: column name from graph object attribute `node_data` to plot a legend (if data added with decorator)
#'@param legend_position  char: legend position
#'@param legend_size      numeric: legend size
#'@param text             char: title text
#'@param text_size        numeric: title size
#'@param text_color       char: title color
#'@param text_font        numeric: title font
#'@param ncol             numeric: number of cols for panel (max 4 panels)
#'@param nrow             numeric: number of rows for panel (max 4 panels)
#'@param dev_off          bool: device off
#'@param ...              any other parameter passed to igraph::pot.igraph
#'
#' @export
plot_netview <- function(g, legend=NULL, legend_position='topleft', legend_size=0.7,
                         text=NULL, text_size=1, text_color='black', text_font=1,
                         ncol=NULL, nrow=NULL, dev_off=F, ...){

  if (igraph::is.igraph(g)) {
    plt_netview(g, legend=legend, legend_position=legend_position, legend_size=legend_size,
                text=text, text_size=text_size, text_color=text_color, text_font=text_font, ...)
  } else {

    if (!is.null(ncol) | !is.null(nrow)) {


      if (!is.null(ncol) & is.null(nrow)) nrow <- ceiling(length(g)/ncol)
      else if (is.null(ncol) & !is.null(nrow)) ncol <- ceiling(length(g)/nrow)
      else if (!is.null(ncol) & !is.null(nrow))  {}# both specified
      else stop('One of: number of panel columns (ncol) or number of panel rows (nrow) must be specified.')

      # if (ncol*nrow > 4 | length(g) > 4) stop('Panel cannot accommodate more than four plots.')

      par(mfrow=c(nrow, ncol))

      }

    sapply(g, plt_netview, legend=legend, legend_position=legend_position, legend_size=legend_size,
           text=text, text_size=text_size, text_color=text_color, text_font=text_font,
           simplify = FALSE, USE.NAMES = TRUE, ...)

  }

  if (!is.null(ncol) | !is.null(nrow)) par(mfrow=c(1,1))

  if (dev_off) dev.off()
}

#' Helper function
#' @keywords internal
#' @noRd
plt_netview <- function(g, legend=NULL, legend_position='topleft', legend_size=0.7, text=NULL, text_size=1, text_color='black', text_font=1, ...){


  if (is.null(igraph::V(g)$label)) igraph::V(g)$label <- rep(NA, igraph::vcount(g))

  if (is.null(igraph::V(g)$pie_data)){

    igraph::plot.igraph(g, vertex.label.font=g$label_settings$font, vertex.label.cex=g$label_settings$size,
                        vertex.label.dist=g$label_settings$dist, vertex.label.color=g$label_settings$color,
                        mark.groups=g$community_settings$groups, mark.col=g$community_settings$color,
                        mark.border=g$community_settings$border, ...)

    if (!is.null(legend)) {
      if(is.null(g$node_data)) stop('No node data has been attached to graph for legend plotting (%@%)')
      if (is.character(legend)) legend_data <- g$node_data[[legend]] else legend_data <- legend

      legend(legend_position, legend=unique(legend_data), fill=unique(igraph::V(g)$color), cex=legend_size, bt='n', col='black')
    }

  } else {

    igraph::plot.igraph(g, vertex.label.font=g$label_settings$font, vertex.label.cex=g$label_settings$size,
                        vertex.label.dist=g$label_settings$dist, vertex.label.color=g$label_settings$color,
                        vertex.shape='pie', vertex.pie=igraph::V(g)$pie_data, vertex.pie.color=igraph::V(g)$pie_color,
                        vertex.pie.border=g$pie_settings$border_color, mark.groups=g$community_settings$groups,
                        mark.col=g$community_settings$color, mark.border=g$community_settings$border, ...)

    if (!is.null(legend)) legend(legend_position, legend=unique(g$pie_settings$name), fill=g$pie_settings$color,
                                 cex=legend_size, bt='n', col='black')
  }


  if (rlang::is_true(text)) title(paste0("k = ", g$k), col.main=text_color, cex.main=text_size, font.main=text_font)
  else if (!is.null(text)) title(as.character(text), col.main=text_color, cex.main=text_size, font.main=text_font)

}

#' K-selection Plot
#'@param graphs    vector/list of igraph objects at increasing k-nearest-neighbor values from netviewr::netview
#'@export
plot_kselect <- function(graphs){

  ## Graphs must be a list of graph objects at increasing k from NetView

  data <- tibble::as_tibble(t(sapply(graphs, function(g){

    if (is.null(g$communities)) stop(paste0('Graph (', 'k = ', g$k, 'is not decorated with communities'))

    community_sizes <- sapply(g$communities, function(com) length(igraph::sizes(com)))

    row <- unlist(append(community_sizes, list('k' = g$k)))

  })))

  mdat <- reshape2::melt(data, id='k')
  names(mdat) <- c('k', 'Method', 'Communities')

  p <- ggplot2::ggplot(data=mdat, ggplot2::aes(x=k, y=Communities, color=Method)) + ggplot2::geom_line(size=1.5)

  return(p)

}

#' Matrix Reader
#'@param file       char: file path to matrix - no header, no rownames
#'@param diag       bool: whether dist matrix is diagonal only
#'@param names      vec: vector of row and column names
#'@param ...        any other parameter passed to `scan` function to read file
#'@export
read_dist <- read_triangular <- function(file="", diag=FALSE, names=paste("X", 1:n, sep=""), ...) {

  ## The following is slightly modified from sem:::read.moments, which is
  ## Copyright 2007 John Fox and is licensed under the GPL V2+

  elements <- scan(file = file, quiet=TRUE, ...)
  m <- length(elements)
  d <- if (diag) 1 else -1
  n <- floor((sqrt(1 + 8 * m) - d) / 2)
  if (m != n * (n + d) / 2) stop("wrong number of elements")
  if (length(names) != n)   stop("wrong number of variable names")
  X <- diag(n)
  X[upper.tri(X, diag=diag)] <- elements
  rownames(X) <- colnames(X) <- names
  X <- X + t(X)
  diag(X) <- diag(X) / 2
  if (!diag) diag(X) <- rep(0, length(diag(X)))
  return(X)
}
