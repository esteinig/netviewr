# netviewr <a href='https://github.com/esteinig'><img src='man/logos/logo_simple.png' align="right" height="200" /></a>

![](https://img.shields.io/badge/lang-R-blue.svg)
![](https://img.shields.io/badge/version-2.1.0-blue.svg)
![](https://img.shields.io/badge/published-MolEcoRes-green.svg)

## Overview

The `netviewr` package offers a set of operators and functions that make 
working with data-driven plots of `igraph` objects more pleasant by:
  
  - decorating graph objects with user data
  - translating data into graph attributes
  - plotting decorated graphs smoothly

`Netviewr` is built with its original application in mind, which is the visualization of genetic population structure and associated meta data from genome-wide single nucleotide polymorphisms (SNPs) for both eukaryotic and prokaryotic species. While it is not a suitable package for __determining__ population structure, it can be used to effectively __visualize__ data (traits, phenotypes, genotypes...) across population structures; it should be used in conjunction with other statistically more sophisticated tools like DAPC or Admixture. 

## Citation

If you are using `netviewr` for research applications, for now please cite:

> Steinig et al. (2016) - Netview P: a network visualization tool to unravel complex population structure using genome‐wide SNPs - Molecular Ecology Resources 16 (1), 216-227

## Table of contents

**`v2.1.0`**

- [Install](#install)
- [General usage](#usage)
  - [Graph decorators](#graph-decorators)
  - [Decorator pipelines](#decorator-pipelines)
  - [Netview plots](#netview-plots)
- [Population genomics](#population-genomics)
  - [Population graphs](#population-graphs)
  - [Community detection](#community-detection)
  - [K-selection plots](#k-selection-plots)
  - [Admixture plots](#admixture-plots)
- [Decorator functions](#decorator-functions)
  - [Node decorators](#node-decorators)
  - [Special decorators](#special-decorators)
  - [Custom decorators](#custom-decorators)
- [Contributions](#contributions)


## Install

``` r

# Install netviewr:
install.packages("netviewr")

# Development version from GitHub:
# install.packages("devtools")
devtools::install_github("esteinig/netviewr")
```
## Usage

### Graph decorators

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

### Decorator pipelines

Besides decorator functions, decorator operators can accept a `data.frame` or `tibble` containing the data to stack on the graph. This
allows for passing the column name to the decorator function:

```r
node_data <- tibble(x=letters[1:10], y=1:10)        # generate 10 x 2 node data tibble

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

Graphs from lists can also be selected using the `magrittr` select operator `%$%` if the list is named, or the `tidyr` function `extract`:

```r
graphs %>% tidyr::extract(1:2) %>% plot_netview()                   # use tidyr extract pipeline
```

### Netview plots

The `plot_netview` function translates the decorated graphs into plots using `igraph::plot.igraph`. Note that all settings provided with decorator functions can be overwritten or refined by passing standard arguments for `igraph::plot.igraph` to `netviewr::plot_netview`. Legends and titles can be set using the `legend` and `title` arguments. If either `ncol` or `nrow` are set, the function expects a list of graphs, which is then translated into a panel view containing up to four plots. Graphs can be piped into the plot function using the `magrittr` operator `%>%` - this allows users to configure graphs by setting a basic configuration, and then change attributes on subsequent graph assignments.

```r
base_graph <- igraph::sample_gnm(n=10, m=15) %@%
              node_data %@%                                      
              node_size(data='y', min=5, max=8) 

base_graph %@% node_color(data='x', palette='BuGn') %>% plot_netview()
base_graph %@% node_color(data='y', palette='PuBu') %>% plot_netview()
```

## Population genomics

`Netview` was originally developed to provide a means of visualizing complex, high-resolution population structure and associated meta-data. In the original publications by Neuditschko et al. (2012) and Steinig et al. (2016) we have shown that these visualizations are quite useful for natural populations (e.g. wolf populations in Alaska) and artificial systems (e.g. pearl oyster pedigrees in aquaculture farms). In subsequent publications, we have also used `Netview` to visualize population structure of bacterial pathogens (e.g. Staphylococcus aureus) and many other cool examples now exist in the literature (e.g. shark hybridisation in the Galapagos).

However, it should be noted that `Netview` is primarily designed to visualize structure from genetic similarity matrices, not determine a 'correct' population structure, or admixture proportions between populations. One of the reasons for this is that we directly use the input distance matrices in the mutual k-nearest-neighbor algorithm where the parameter *k* can be selected to unravel the genetic structure in the network topologies at *any* resolution: a smaller *k* results in many disjointed groups of individuals (high resolution) and a larger *k* causes the topologies to become more heterogenous (low resolution). As such, we always recommend to run other, statistically more sophisticated software for population structure determination (like DAPC. Structure or Admixture) in parallel with `Netview`. 

However, we also now provide a *k* selection method, which can be used to determine an 'optimal' value for *k* based on the congruence of cluster (community) detection algorithms run over a range of *k* values (see below). Community detection algorithms can also be used to demarcate and sort individuals into groups determined from the structure in the network topologies.

`Netviewr` provides specialised decorator functions for these genetic applications, including pie charts for admixture proportions of each inidvidual sample (e.g. derived from `Admixture`) and classification of individuals into distinct populations using community-detection algorithms.

### Population graphs

**Input distance matrix**

Before using `Netview` to obtain a mutual *k*-nearest-neighbor graph, a distance matrix should be computed that is suitable for your type of data. For example, for eukaryotic SNP panels, you may want to use the `1-IBS` matrix that can be computed in `PLINK`. For others applications, e.g. in bacterial pathogen whole genome sequence data, you may want to compute a simple SNP distance matrix from reference alignments, e.g. with the excellent Rust package [`psdm`](https://github.com/mbhall88/psdm). You can also use a phylogenetic tree as the basis for your distance matrix, e.g. by computing pairwise root-to-tip distance between samples, or even a completely non-genetic distance measure! 

Ideally the input matrix has few missing data, as these may bias the mutal *k*-nearest-neighbor algorithms to find similarity between individuals with shared missing sites. Data on which the distances are based should be reasonably high resolution (i.e. SNPs not microsatellites) as ultimately, the mutual *k*-nearest-neighbor algorithm is a 'dumb' machine learning approach and requires suitable high resolution data to work with in the first place.

**Population graph inference**

Simple graph construction from distance matrix file (symmetrical) over a range of *k* and default plotting:

```r
netviewr::read_dist("dist.tsv", sep="\t") %>% netviewr::netview(k=20) %>% netviewr::plot_netview()
```

This pipeline also works without piping:

```r
dist_matrix <- netviewr::read_dist("dist.tsv", sep="\t")
graph <- netviewr::netview(dist_matrix, k=20)
plot_netview(graph)
```


**Graph plot interpretation**

Nodes that share many edges (mutal nearest neighbors) tend to cluster together in the default network visualization algorithm (`Fruchterman-Reingold`). It needs to be stressed that the overall layout of the graph **does not hold any interpretaive value** - that is clusters that are not connected or connected only by few edges can be rearranged by the layout algorithm, so that two closely located but unconnected clusters of nodes, do **not** indicate genetic similarity. This information is exclusively determined by the edges.

For example:

...

### Community detection

...

### K-selection plots

...

### Admixture plots

...

## Decorator functions

Decorator functions modify the graph object by translating data into graph attributes and attaching it to the appropriate slots in the graph object. These are eventually recognized by the `plot_netview` function so that graphs can be constructed and decorated independently of the `igraph::plot.igraph` arguments, which can be a little arcane in their original implementation. 

### Node decorators

##

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

### Custom decorators

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

### Geonet

Geographical decorators and projections *under development*

## Contributions

We welcome any and all suggestions or pull requests. Please feel free to open an issue in the repository on `GitHub`.


