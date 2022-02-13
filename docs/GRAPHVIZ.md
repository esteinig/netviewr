# General graph visualization 

## Graph decorators

The workhorse of the package is the decorator operator `%@%`. It accepts an `igraph` object (left side) and pipes it 
into a decorator function (right side). Decorator functions transform data into graph attributes and attach them
to the correct slots in the graph object for use in the plot function. Several decorator functions in sequence can
stack their outputs onto the graph. The pipeline ends with the `magrittr` pipe operator `%>%` to feed the decorated graph
into the plot function:

```r
g <- igraph::sample_gnm(n=10, m=15) %@%                      
     node_color(data=letters[1:10], palette='BuGn') %@%      
     node_size(data=1:10, min=5, max=8)                     

g %>% plot_netview()                
```


**Note** that an alias for the graph decorator operator is `%g%` if you need to avoid the namespace clash with `purrr::%@%` which is an attribute assignment operator.

## Decorator pipelines

Besides decorator functions, decorator operators can accept a `data.frame` or `tibble` containing the data to stack on the graph. This
allows for passing the column name to the decorator function:

```r
node_data <- tibble::tibble(x=letters[1:10], y=1:10)        # generate 10 x 2 node data tibble

g <- igraph::sample_gnm(n=10, m=15) %@%             # generate random graph with 10 nodes
     node_data %@%                                  # decorate graph with node data tibble
     node_color(data='x', palette='BuGn') %@%       # decorate nodes with colors paletted by x
     node_size(data='y', min=5, max=8)              # decorate nodes with values rescaled by y
     
g %>% plot_netview()                                # plot decorated graph from magrittr pipe
```

The pipeline can also be executed on a list of graphs: 

```r

graphs <- lapply(1:2, function(x) igraph::sample_gnm(n=10, m=15)) %@%
          node_data %@%                                                 
          node_color(data='x', palette='PuOr') %@%                      
          node_size(data='y', min=15, max=20)                          
     
graphs %>% plot_netview(legend='x')                     
```

## Netview plots

The `plot_netview` function translates the decorated graphs into plots using `igraph::plot.igraph`. Note that all settings provided with decorator functions can be overwritten or refined by passing standard arguments for `igraph::plot.igraph` to `netviewr::plot_netview`. Graphs can be piped into the plot function using the `magrittr` operator `%>%` - this allows users to configure graphs by setting a basic configuration, and then change attributes on subsequent graph assignments.

```r
base_graph <- igraph::sample_gnm(n=10, m=15) %@%
              node_data %@%                                      
              node_size(data='y', min=5, max=8) 

base_graph %@% node_color(data='x', palette='BuGn') %>% plot_netview()
base_graph %@% node_color(data='x', palette='PuBu') %>% plot_netview()
```

Legends and titles can be set using the `legend` and `text` arguments. `Legend` parameter is character that selects a column from attached node data when `%@%` is used (see above)

```r
graphs %>% plot_netview(legend='x', text="Test", text_size=1.5)
```

