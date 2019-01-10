# netviewr <a href='https://github.com/esteinig'><img src='man/figures/logo.png' align="right" height="139" /></a>

![](https://img.shields.io/badge/CRAN-0.1-green.svg)
![](https://img.shields.io/badge/docs-latest-green.svg)
![](https://img.shields.io/badge/lifecycle-maturing-blue.svg)

## Overview

The `netviewr` package offers a set of operators and functions that make 
data-driven plotting of `igraph` objects more pleasant by:
  
  - decorating graph objects with user data
  - translating data into graph attributes
  - stacking data for decorator functions
  - providing geographical projections

##

The workhorse of the package is the decorator operator `%@%`, which expects on the left side
an `igraph` object and on the right side a decorator function. Decorator functions transform 
data into graph attributes and attach them to the correct slots in the graph object for plotting. 
Decorator operators therefore enables a tidyverse pipeline syntax to decorate graph objects. The
pipeline ends with the `magrittr` pipe operator `%>%` to feed the decorated graph into the plot
function:

```r
g <- igraph::sample_gnm(n=10, m=15) %@%                       # generate random graph with 10 nodes
     node_color(data=letters[1:10], palette='BuGn') %@%       # decorate nodes with colors paletted by letters
     node_size(data=1:10, min=5, max=8) %>%                   # decorate nodes with values rescaled by integers
     plot_netview()                                           # plot decorated graph from magrittr pipe
```

The decorator operator can accept a `data.frame` or `tibble` containing the data to stack on the graph. This
allows for passing the column name to the decorator function:

```r
node_data <- tibble(x=letters[1:10], y=1:10)        # generate 10 x 2 node data tibble

g <- igraph::sample_gnm(n=10, m=15) %@%             # generate random graph with 10 nodes
     node_data %@%                                  # decorate graph with node data tibble
     node_color(data='x', palette='BuGn') %@%       # decorate nodes with colors paletted by x
     node_size(data='y', min=5, max=8) %>%          # decorate nodes with values rescaled by y
     plot_netview()                                 # plot decorated graph from magrittr pipe
```
The pipeline can also be executed on a list of graphs, here we add labels and pipe the list of decorated graphs
into a panel view with legends:

```r

g <- lapply(1:2, function(x) igraph::sample_gnm(n=10, m=15)) %@%   # generate list of random graphs
     node_data %@%                                                 # decorate graph with node data tibble
     node_color(data='x', palette='PuOr') %@%                      # decorate graphs with node colors
     node_label(data='x', color='black', size=0.8) %@%             # decorate graphs with node labels
     node_size(data='y', min=15, max=20) %>%                       # decorate graphs with node sizes
     plot_netview(legend='x', ncol=2)                              # plot decorated graphs in panel
```

## Installation

``` r

# Install netviewr:
install.packages("netviewr")

# Development version from GitHub:
# install.packages("devtools")
devtools::install_github("esteinig/netviewr")
```
## Usage

