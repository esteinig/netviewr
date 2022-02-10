# Decorator functions

Decorator functions modify the graph object by translating data into graph attributes and attaching it to the appropriate slots in the graph object. These are eventually recognized by the `plot_netview` function so that graphs can be constructed and decorated independently of the `igraph::plot.igraph` arguments, which can be a little arcane in their original implementation. 

## Node decorators

```r
node_color(g, data=NULL, condition=NULL, palette='BuGn', color='gray', opacity=1, n_color=NULL)
```

Map string or factorial data to node colors from a `palette` or set a uniform `color` if no data is provided.

<img src='man/plots/color_1.png' height="300" /> <img src='man/plots/color_2.png' height="300" />

##

```r
node_size(g, data=NULL, size=4,  min=5, max=10)
```

Map numeric data to node sizes, rescale the data to a range of `min` and `max` sizes, or set a uniform `size` if no data is provided.

<img src='man/plots/size_1.png' height="300" /> <img src='man/plots/size_2.png' height="300" />

##

```r
node_label(g, data=NULL, label=NA, size=0.8, color='black', family='serif', font=1, dist=0)
```

Map string data as labels or set a uniform `label` if no data is provided. Labels can be somewhat refined with arguments in this decorator, but more sophisticated options are available in `igraph::plot.igraph`.

<img src='man/plots/labels_1.png' height="300" /> <img src='man/plots/labels_2.png' height="300" />

##

```r
node_shape(g, data=NULL, shape=c('circle', 'square', ...))
```

Map string or factorial data to a node shape, recycles values if more data levels available than shapes.

<img src='man/plots/shape_1.png' height="300" /> <img src='man/plots/shape_2.png' height="300" />

### Special decorators

##

```r
node_pie(g, data=NULL, palette='BuGn', n_color=NULL, border_color='black', match_func=dplyr::starts_with)
```

Map proportional data values over `m` categories (must sum to 1) to nodes that represent pie graphs, where each slice is colored according to `palette`. Data can be a `tibble`, `data.frame` or string column name, in which case the data argument uses the `match_func` to select the columns from the data the graph was decorated with. For instance, if all columns that hold the pie graph values start with `pie`, such as `pie_1`, `pie_2` and `pie_3` the data argument would be `data = 'pie'`.

<img src='man/plots/pie_1.png' height="300" />

##

```r
community(g, method='walktrap', polygon=NULL, palette='PuBuGn', opacity=1, border=NA, n_color=NULL, ...)
```

Detect communities in the graph and draw polygons colored by `palette` around the clusters in the network topology. Multiple community detection methods can be specified. If `polygon` is a boolean and `TRUE` the first method is used to draw polygons, otherwise the polygon accepts a string specifying which community detections results should be drawn, e.g. `polygon = 'walktrap'`. Any other named arguments that specify the behaviour of the community detection algorithm can be passed via the ellipsis argument (`...`). Community detection can be run before assigning other node attributes. The name of the community detection algorithm then maps community membership to node attributes, for example:

```r
g <- igraph::sample_gnm(n=10, m=15) %@% 
     community(method='walktrap') %@%
     node_color(data='walktrap', palette='BuGn')
```

<img src='man/plots/community_1.png' height="300" /> <img src='man/plots/community_2.png' height="300" />

## Custom decorators

Decorators are functions that require a graph object (`g`) and return an anonymous function (`func`) that first modifies and then returns the graph object. A simple example would be a decorator that assigns node colors via a `colorize` function. Without error checks and other essential behaviour and for demonstration purpose only, this could then be written as:

```r
node_color <- function(g, data=NULL, palette='BuGn', ...){
  func <- (g, ...) {
    V(g)$color <- colorize(data, palette)
    return(g)  
  }
  return(func)
}
```

## Geonet

Geographical decorators and projections *under development*
