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

> Netviewr implements the same tasks and methods as the published version of NetView - except in modern R. If the new syntax feels weird, or you need backward compatibility, the [old version](https://github.com/esteinig/netview/blob/master/README_OLD.md) is still accessible. You can also use all functions in base syntax (without pipes) as shown in the [population graphs](docs/POPGENOM.md#population-graphs) examples.

## Citation

If you are using `netviewr` for research applications, for now please cite:

> Steinig et al. (2016) - Netview P: a network visualization tool to unravel complex population structure using genome‐wide SNPs - Molecular Ecology Resources 16 (1), 216-227

If you have trouble accessing the paper, have a look in the [documentation](docs/) folder.

## Table of contents

**`v2.1.0`**

- [Install](#install)
- [Population genomics](docs/POPGENOM.md#population-genomics)
  - [Population graphs](docs/POPGENOM.md#population-graphs)
  - [Community detection](docs/POPGENOM.md#community-detection)
  - [K-selection plots](docs/POPGENOM.md#k-selection-plots)
  - [Admixture plots](docs/POPGENOM.md#admixture-plots)
- [Graph visualization](docs/GRAPHVIZ.md#general-graph)
  - [Graph decorators](docs/GRAPHVIZ.md#graph-decorators)
  - [Decorator pipelines](docs/GRAPHVIZ.md#decorator-pipelines)
  - [Netview plots](docs/GRAPHVIZ.md#netview-plots)
- [Decorator functions](docs/DECFUNC.md#decorator-functions)
  - [Node decorators](docs/DECFUNC.md#node-decorators)
  - [Special decorators](docs/DECFUNC.md#special-decorators)
  - [Custom decorators](docs/DECFUNC.md#custom-decorators)
- [Contributions](#contributions)


## Install

``` r

# Install netviewr (not on CRAN yet)
# install.packages("netviewr")

# Development version from GitHub:
# install.packages("devtools")
devtools::install_github("esteinig/netviewr")
```

## Population graph example

Build and plot a mutual k-nearest-neighbor graph from a (genetic) distance matrix.

```r
matrix(rnorm(600), nrow=30) %>% netviewr::netview(k=20) %>% netviewr::plot_netview()
```

## General graph example

Annotate a graph with data and plot the graph.

```r
node_data <- tibble::tibble(x=letters[1:10], y=1:10)    # generate 10 x 2 node data tibble

g <- igraph::sample_gnm(n=10, m=15) %@%                 # generate random graph with 10 nodes
     node_data %@%                                      # decorate graph with node data tibble
     node_color(data='x', palette='BuGn') %@%           # decorate nodes with colors paletted by x
     node_size(data='y', min=5, max=8)                  # decorate nodes with values rescaled by y
     
g %>% plot_netview()                                    # plot decorated graph from magrittr pipe 
```

## Plot examples

<img src='man/plots/color_1.png' height="300" /> <img src='man/plots/size_2.png' height="300" />
<img src='man/plots/labels_2.png' height="300" /> <img src='man/plots/shape_2.png' height="300" />
<img src='man/plots/community_1.png' height="300" /> <img src='man/plots/pie_1.png' height="300" />

## Contributions

We welcome any and all suggestions or pull requests. Please feel free to open an issue in the repository on `GitHub`.


