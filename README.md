# netviewr <a href='https://github.com/esteinig'><img src='man/figures/logo.png' align="right" height="139" /></a>

![](https://img.shields.io/badge/CRAN-0.1-green.svg)
![](https://img.shields.io/badge/docs-latest-green.svg)
![](https://img.shields.io/badge/lifecycle-maturing-yellow.svg)

## Overview

The `netviewr` package offers a set of operators and functions that make 
working and plotting of `igraph` objects more pleasant by:
  
  - decorating graph objects with user data
  - translating data into graph attributes
  - stacking data for decorator functions
  - providing geographical projections

---

The main workhorse of the packages is the graph decorator operator `%@%`, which expects on the left side
an `igraph` object and on the right side a `data.frame` or `tibble` to attach to the graph. Data columns 
can then be used to further decorate the graph with decorator functions. Decorator functions transform the
data into graph attributes and attach them to the correct slots in the graph object for plotting. 

The decorator operator therefore enables a pipeline syntax to decorate graph objects, additionally using
the standard `magrittr` pipe operator `%>%` to feed the decorated graph into the plot function:

```r
node_data <- tibble(x=letters[1:10], y=1:10)        # generate 10 x 2 node data tibble

g <- igraph::sample_gnm(n=10, m=15) %@%             # generate random graph with 10 nodes
     node_data %@%                                  # decorate graph with node data tibble
     node_color(data='x', palette='BuGn') %@%       # decorate nodes with colors paletted by x
     node_size(data='y', min=5, max=8) %>%          # decorate nodes with values rescaled by y
     plot_netview()                                 # plot decorated graph from magrittr pipe
```

## Installation

``` r

# Install netviewr:
install.packages("netviewr")

# Development version from GitHub:
# install.packages("devtools")
devtools::install_github("esteinig/netviewr")
```
