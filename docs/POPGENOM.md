# Population genomics
`Netview` was originally developed to provide a means of visualizing complex, high-resolution population structure and associated meta-data. In the original publications by [Neuditschko et al. (2012)]() and [Steinig et al. (2016)]() we demonstrated that these visualizations can be quite useful to unravel natural population structures (e.g. wolf populations) and pedigree structures (e.g. pearl oysters). In subsequent publications, we have also used `Netview` to visualize population structure of bacterial pathogens (e.g. [*Staphylococcus aureus*]()) and many other cool examples now exist in the literature (e.g. [Koala populations](https://doi.org/10.1007/s10592-015-0784-3) or [shark hybridisation](https://doi.org/10.1007/s10592-017-0967-1)).

`Netview` is primarily designed to visualize structure from genetic similarity matrices, not determine a 'correct' population structure. One of the reasons for this is that we directly use the distance matrix in the mutual k-nearest-neighbor algorithm, where the number of mutual nearest-neighbors parameter (*k*) can be chosen to unravel structure at *any* resolution: a smaller value results in many disjointed groups of individuals ("high resolution") and a larger value causes the topologies to become more heterogenous ("low resolution"). We always run statistically more sophisticated software for population structure determination (like `Structure` or `Admixture`) in parallel with `Netview`. 

We do provide a selection method, which can be used to determine an 'optimal' value for the number of mutal nearest-neighbors based on the congruence of community (cluster) detection algorithms run over a range of parameter values (see below). Please note that this is not a biological optimisation i.e. the congruence does not tell us anything about the optimal number of populations in the data. Instead, it tells us at what resolution the network *assembles* into a coherent topology. However, in our experience, this often coincides with the optimal number of populations inferred using other population genomic approaches. 

`Netviewr` provides specialised decorator functions for these genetic applications, including pie charts for admixture proportions of each inidvidual sample (e.g. derived from `Admixture`). Community detection algorithms can be used to demarcate and sort individuals into groups determined from the structure in the network topologies.

## Population graphs

### Input distance matrix

Before using `Netview` to obtain a mutual *k*-nearest-neighbor graph, a distance matrix should be computed that is suitable for your type of data. For example, for eukaryotic SNP panels, you may want to use the `1-IBS` matrix that can be computed in `PLINK`. For others applications, e.g. in bacterial pathogen whole genome sequence data, you may want to compute a simple SNP distance matrix from reference alignments, e.g. with the excellent Rust package [`psdm`](https://github.com/mbhall88/psdm). You can also use a phylogenetic tree as the basis for your distance matrix, e.g. by computing pairwise root-to-tip distance between samples, or even using a non-genetic distance measure of similarity.

Ideally the input matrix has few missing data, as these may bias the mutal nearest-neighbor algorithm to find similarity between individuals with shared missing sites. Data on which the distances are based should be reasonably high resolution (i.e. SNPs, not microsatellites) as ultimately we employ a 'dumb' machine learning approach which requires suitable high resolution 

### Population graph inference

Let's create a series of mutual k-nearest-neighbor graph from random distances between 20 samples:

```r
dist <- matrix(rnorm(400),nrow=20)
```

We can then pipe the matrix into the graph builder, noting that the highest value of `k` is `n-1`:

```r
dist %>% netview(k=1:19)
```

Let's say we have a dataframe specifying some values for each sample, which we want to plot as node colors:

```r
node_data <- data.frame(some_data=letters[1:20])
```

We can now decorate (`%@%`) the graphs with these data and the `node_color` decorator to map the values to colors:

```r
g <- dist %>% netview(k=1:19) %@% node_data %@% node_color(data="some_data", palette="BuGnYl")
```

Plot the decorated graphs individually or as a panel:

```r
 g %>% plot_netview()                 # individual plots
 g %>% plot_netview(nrow=4, ncol=5)   # panel plots
```

Graph construction from a distance matrix file (symmetrical or triangular, without column or row names) over a single pareameter value and using default plotting:

```r
netviewr::read_dist("dist.tsv", sep="\t") %>% netviewr::netview(k=20) %>% netviewr::plot_netview()
```

Of course this pipeline also works without piping:

```r
dist_matrix <- netviewr::read_dist("dist.tsv", sep="\t")
graph <- netviewr::netview(dist_matrix, k=20)
netviewr::plot_netview(graph)
```

### Population data visualization

`Netviewr` can be used along with a `data.frame` or `tibble` to overlay data on the nodes (indivudals) in the graph. For example, you may want to highlight admixture proportions of each individual across the network topology, or investigate the assignment of predefined populations (e.g. pedigrees) compared with the genetic population structure.

Associated data must have the same number of rows as there are individuals in the network and **in the same order** as rows in the input distance matrix.

```r



```


### Graph plot interpretation

Nodes that share many edges (mutal nearest neighbors) tend to cluster together in the default network visualization algorithm (`Fruchterman-Reingold`). It needs to be stressed that the overall layout of the graph **does not hold any interpretaive value** - that is clusters that are not connected or connected only by few edges can be rearranged by the layout algorithm, so that two closely located but unconnected clusters of nodes, do **not** indicate genetic similarity. This information is exclusively determined by the edges.

For example, note the oreientation of the graph and location of the disconnected clusters in theaw two **equivalent** graphs:

<img src='../man/plots/color_1.png' height="300" /> <img src='../man/plots/size_1.png' height="300" /> 

## Community detection

...

## K-selection plots

...

### Admixture plots

...
