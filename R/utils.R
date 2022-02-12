#'@param g                
#'@param legend  
#'@param legend_position         
#'@param legend_size                  
#'@param text               
#'@param text_size      
#'@param text_color         
#'@param text_font                 
#'@param ncol               
#'@param nrow                  
#'@param dev_off               
#'@param ...          

#'@usage 
#'
#'@return 

#' @export
plot_netview <- function(g, legend=NULL, legend_position='topleft', legend_size=0.7, 
                         text=NULL, text_size=1, text_color='black', text_font=1, 
                         ncol=NULL, nrow=NULL, dev_off=F, ...){
  
  if (is.igraph(g)) {
    plt_netview(g, legend=legend, legend_position=legend_position, legend_size=legend_size,
                text=text, text_size=text_size, text_color=text_color, text_font=text_font, ...)
  } else {
    
    if (!is.null(ncol) | !is.null(nrow)) {
      
    
      if (!is.null(ncol) & is.null(nrow)) nrow <- ceiling(length(g)/ncol)
      else if (is.null(ncol) & !is.null(nrow)) ncol <- ceiling(length(g)/nrow)
      else stop('One of: number of panel columns (ncol) or number of panel rows (nrow) must be specified.')
      
      if (ncol*nrow > 4 | length(g) > 4) stop('Panel cannot accommodate more than four plots.')
      
      par(mfrow=c(nrow, ncol))
    
      }
    
    sapply(g, plt_netview, legend=legend, legend_position=legend_position, legend_size=legend_size, 
           text=text, text_size=text_size, text_color=text_color, text_font=text_font,
           simplify = FALSE, USE.NAMES = TRUE, ...)
    
  }
  
  if (!is.null(ncol) | !is.null(nrow)) par(mfrow=c(1,1))
  
  if (dev_off) dev.off()
}


plt_netview <- function(g, legend=NULL, legend_position='topleft', legend_size=0.7, text=NULL, text_size=1, text_color='black', text_font=1, ...){
  
  
  if (is.null(V(g)$label)) V(g)$label <- rep(NA, vcount(g))
  
  if (is.null(V(g)$pie_data)){
    
    igraph::plot.igraph(g, vertex.label.font=g$label_settings$font, vertex.label.cex=g$label_settings$size, 
                        vertex.label.dist=g$label_settings$dist, vertex.label.color=g$label_settings$color,
                        mark.groups=g$community_settings$groups, mark.col=g$community_settings$color,
                        mark.border=g$community_settings$border, ...)
    
    if (!is.null(legend)) {
      if (is_string(legend)) legend_data <- g$node_data[[legend]] else legend_data <- legend
      
      legend(legend_position, legend=unique(legend_data), fill=unique(V(g)$color),
                                 cex=legend_size, bt='n', col='black')
    }
    
  } else {
    
    igraph::plot.igraph(g, vertex.label.font=g$label_settings$font, vertex.label.cex=g$label_settings$size, 
                        vertex.label.dist=g$label_settings$dist, vertex.label.color=g$label_settings$color,
                        vertex.shape='pie', vertex.pie=V(g)$pie_data, vertex.pie.color=V(g)$pie_color, 
                        vertex.pie.border=g$pie_settings$border_color, mark.groups=g$community_settings$groups,
                        mark.col=g$community_settings$color, mark.border=g$community_settings$border, ...)
    
    if (!is.null(legend)) legend(legend_position, legend=unique(g$pie_settings$name), fill=g$pie_settings$color,
                                 cex=legend_size, bt='n', col='black')
  }
  
  
  if (is_true(text)) title(paste0("k = ", g$k), col.main=text_color, cex.main=text_size, font.main=text_font)
  else if (!is.null(text)) title(as.character(text), col.main=text_color, cex.main=text_size, font.main=text_font)
  
}


#'@param ...          

#'@usage 
#'
#'@return 

#' @export
plot_kselect <- function(graphs){
  
  ## Graphs must be a list of graph objects at increasing k from NetView
  
  data <- as_tibble(t(sapply(graphs, function(g){
    
    if (is.null(g$communities)) stop(paste0('Graph (', 'k = ', g$k, 'is not decorated with communities'))

    community_sizes <- sapply(g$communities, function(com) length(sizes(com)))
    
    row <- unlist(append(community_sizes, list('k' = g$k)))
    
  })))
  
  mdat <- melt(data, id='k')
  names(mdat) <- c('k', 'Method', 'Communities')
  
  p <- ggplot(data=mdat, aes(x=k, y=Communities, color=Method)) + geom_line(size=1.5)
  
  return(p)
  
}

#'@param ...
#'@param ...
#'@param ...          

#'@usage 
#'
#'@return 

#' @export
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