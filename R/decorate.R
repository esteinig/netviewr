#' Example function
#' @keywords internal
#' @noRd
decorate_nodes <- function(g, data=NULL, func=NULL, ...){

  # General node decorator format example:

  func <- function(g, ...){

    # If column name, data vector is column:
    if (is.character(data) & length(data) == 1) data <- g$node_data[[data]]

    # Hard coded test for data, could be NULL, need to change:
    if (is.null(data)) stop('Valid column name or data vector must be supplied to graph decorator function.')

    # Function that modifies and returns graph, use data in function scope:
    g <- func(g, ...)

    return(g)
  }
  return(func)
}


#' @export
node_color <- function(g, data=NULL, condition=NULL, palette='BuGn', color='gray', opacity=1, n_color=NULL) {

  func <- function(g, ...){

    if (!igraph::is.igraph(g)) stop('Input must be a graph')

    if (is.null(data)) {

      igraph::V(g)$color <- rep(color, igraph::vcount(g))

    } else {

      if (is.character(data) & length(data) == 1) data <- g$node_data[[data]]

      if (is.character(data)) data <- as.factor(data)
      if (!is.factor(data)) stop('Data must be character or factor.')

      if (is.function(condition)) {
        igraph::V(g)$color <- dplyr::if_else(condition(data), colorize(data, palette=palette, n_col=n_color, alpha=opacity), color)
      } else {
        igraph::V(g)$color <- colorize(data, palette=palette, n_col=n_color, alpha=opacity)
      }
    }
    return(g)
  }
  return(func)
}

#' @export
node_size <- function(g, data=NULL, size=1, min=1, max=4) {

  func <- function(g, ...){

    if (!igraph::is.igraph(g)) stop('Input must be a graph')
    if (is.character(data) & length(data) == 1) data <- g$node_data[[data]]

    if (is.null(data)) {

      igraph::V(g)$size <- rep(size, igraph::vcount(g))

    } else {

      if (!is.numeric(data)) stop('Data must be numeric.')

      igraph::V(g)$size <- scales::rescale(data, to=c(min, max))
    }
    return(g)
  }
  return(func)
}

#' @export
node_shape <- function(g, data=NULL, shape=c('circle', 'square', 'csquare', 'vrectangle', 'rectangle', 'crectangle')) {

  func <- function(g, ...){

    if (!igraph::is.igraph(g)) stop('Input must be a graph')

    if (is.character(data) & length(data) == 1) data <- g$node_data[[data]]
    if (is.null(data)) stop('Valid column name or data vector must be supplied to color_nodes()')

    # Data must be character or factor:

    if (is.character(data)) data <- as.factor(data)
    if (!is.factor(data)) stop('Data must be character or factor.')

    # Factor levels must be smaller than number of shapes, otherwise recycled.

    if (length(shape) < length(levels(data))) shape <- rep(shape, length.out=length(levels(data)))
    else shape <- shape[1:length(levels(data))]

    igraph::V(g)$shape <- as.character(plyr::mapvalues(data, from=levels(data), to=shape))

    return(g)

  }
  return(func)
}

#' @export
node_label <- function(g, data=NULL, label=NA, size=0.8, color='black', family='serif', font=1, dist=0) {

  func <- function(g, ...){

    if (!igraph::is.igraph(g)) stop('Input must be a graph')

    if (is.character(data) & length(data) == 1) data <- g$node_data[[data]]

    if (is.null(data)) {

      igraph::V(g)$label <- rep(label, igraph::vcount(g))

    } else {

      if (!is.character(data)) stop('Data must be character.')

      igraph::V(g)$label<- data
    }

    g$label_settings <- list(
      size = size, dist = dist, font = font,
      color = color, family = family
    )

    return(g)
  }
  return(func)
}

#' @importFrom magrittr "%>%"
#' @export
node_pie <- function(g, data=NULL, palette='BuGn', n_color=NULL, border_color='black', match_func=dplyr::starts_with){

  func <- function(g, ...){

    if (!igraph::is.igraph(g)) stop('Input must be a graph')

    if (tibble::is_tibble(data) | is.data.frame(data)) pie_data <- data()
    else pie_data <- dplyr::select(g$node_data, match_func(data))

    igraph::V(g)$pie_data <- pie_data %>% purrr::pmap(c, use.names = F)
    igraph::V(g)$pie_color = lapply(igraph::V(g)$pie_data, function(x) colorize(data=names(pie_data), palette=palette, n_col=n_color))

    g$pie_settings <- list(
      name = names(pie_data),
      color = colorize(data=names(pie_data), palette=palette, n_col=n_color),
      border_color = border_color
    )

    return(g)
  }
  return(func)
}

#' @importFrom magrittr "%>%"
#' @export
community <- function(g, method='walktrap', polygon=NULL, palette='PuBuGn', opacity=1, border=NA, n_color=NULL, ...){

  func <- function(g, ...){

    if (!igraph::is.igraph(g)) stop('Input must be a graph')

    if (is.logical(polygon) & isTRUE(polygon)) polygon <- method[1]

    algorithms <- sapply(method, function(m) paste0('cluster_', m))

    args <- list(...)

    args$graph <- g

    # An error is raised here when using sapply / lapply,
    # uses simple loop instead:

    comms <- list()
    members <- list()

    for (algorithm in method){
      com <- do.call(paste0('cluster_', algorithm), args=args)
      comms[[algorithm]] <- com
      members[[algorithm]] <- igraph::membership(com)
    }

    members <- tibble::as_tibble(members) %>% dplyr::mutate_if(is.double, as.factor)

    # add to community slot

    if (is.null(g$communities)) g$communities <- comms
    else g$communities <- g$communities %>% append(comms)

    if (is.null(g$node_data)) g$node_data <- members else g$node_data <- dplyr::bind_cols(g$node_data, members)

    if(!is.null(polygon) & !is.logical(polygon) & length(polygon) == 1 ){
      groups <- igraph::communities(comms[[polygon]])

      g$community_settings <- list(
        groups = groups,
        border = border,
        color = colorize(data=seq_along(groups), palette=palette, n_col=n_color, alpha=opacity)
      )
    }

    return(g)
  }
  return(func)
}

#' Helper function
#' @importFrom magrittr "%>%"
#' @keywords internal
#' @noRd
colorize <- function(data=NULL, palette='BuGn', n_col=NULL, r=NULL, alpha=1, verbose=T){

  if (!is.null(data)) {
    if (is.character(data) | is.factor(data) ){
      data <- as.factor(data)
      col <- get_col(palette, n_col)
      col <- colorRampPalette(col)(levels(data) %>% length)
      col <- plyr::mapvalues(data, from=levels(data), to=col)
    }
    else if (is.numeric(data)) {
      col <- get_col(palette, n_col)
      col <- colorRampPalette(col)(length(data))
    }
    else {
      col <- NULL
    }
  } else {
    col <- get_col(palette, n_col)
    if (!is.null(r)) col <- colorRampPalette(col)(r)
  }

  return(adjustcolor(as.character(col), alpha.f=alpha))
}

#' Helper function
#' @importFrom magrittr "%>%"
#' @keywords internal
#' @noRd
get_col <- function(palette, n_col){
  if (is.character(palette) & length(palette) == 1) {

    if (!palette %in% rownames(RColorBrewer::brewer.pal.info)) stop('Palette is not available in RColorBrewer.')

    if (is.null(n_col)) {
      n_col <- levels(data) %>% length
      if (n_col < 3) n_col <- 3
      if (n_col > RColorBrewer::brewer.pal.info[palette, 'maxcolors']){
        n_col <- RColorBrewer::brewer.pal.info[palette, 'maxcolors']
      }
    }
    return(RColorBrewer::brewer.pal(n_col, palette))
  } else if (is.character(palette) & length(palette) > 1) {
    return(palette)
  } else {
    stop('Palette must be single palette name or vector of color strings.')
  }
}
