# Population genomics

`Netview` was originally developed to provide a means of visualizing complex, high-resolution population structure and associated meta-data. In the original publications by Neuditschko et al. (2012) and Steinig et al. (2016) we have shown that these visualizations are quite useful for natural populations (e.g. wolf populations in Alaska) and artificial systems (e.g. pearl oyster pedigrees in aquaculture farms). In subsequent publications, we have also used `Netview` to visualize population structure of bacterial pathogens (e.g. Staphylococcus aureus) and many other cool examples now exist in the literature (e.g. shark hybridisation in the Galapagos).

However, it should be noted that `Netview` is primarily designed to visualize structure from genetic similarity matrices, not determine a 'correct' population structure, or admixture proportions between populations. One of the reasons for this is that we directly use the input distance matrices in the mutual k-nearest-neighbor algorithm, where the number of mutual nearest-neighbors parameter (*k*) can be chosen to unravel structure at *any* resolution: a smaller value results in many disjointed groups of individuals ("high resolution") and a larger value causes the topologies to become more heterogenous ("low resolution"). We always run statistically more sophisticated software for population structure determination (like Structure or Admixture) in parallel with `Netview`. 

We do now provide a selection method, which can be used to determine an 'optimal' value for the number of mutal nearest-neighbors based on the congruence of community (cluster) detection algorithms run over a range of parameter values (see below). Community detection algorithms can be used to demarcate and sort individuals into groups determined from the structure in the network topologies.

`Netviewr` provides specialised decorator functions for these genetic applications, including pie charts for admixture proportions of each inidvidual sample (e.g. derived from `Admixture`).

## Population graphs

### Input distance matrix

Before using `Netview` to obtain a mutual *k*-nearest-neighbor graph, a distance matrix should be computed that is suitable for your type of data. For example, for eukaryotic SNP panels, you may want to use the `1-IBS` matrix that can be computed in `PLINK`. For others applications, e.g. in bacterial pathogen whole genome sequence data, you may want to compute a simple SNP distance matrix from reference alignments, e.g. with the excellent Rust package [`psdm`](https://github.com/mbhall88/psdm). You can also use a phylogenetic tree as the basis for your distance matrix, e.g. by computing pairwise root-to-tip distance between samples, or even a completely non-genetic distance measure! 

Ideally the input matrix has few missing data, as these may bias the mutal *k*-nearest-neighbor algorithms to find similarity between individuals with shared missing sites. Data on which the distances are based should be reasonably high resolution (i.e. SNPs not microsatellites) as ultimately, the mutual *k*-nearest-neighbor algorithm is a 'dumb' machine learning approach and requires suitable high resolution data to work with in the first place.

### Population graph inference

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


### Graph plot interpretation

Nodes that share many edges (mutal nearest neighbors) tend to cluster together in the default network visualization algorithm (`Fruchterman-Reingold`). It needs to be stressed that the overall layout of the graph **does not hold any interpretaive value** - that is clusters that are not connected or connected only by few edges can be rearranged by the layout algorithm, so that two closely located but unconnected clusters of nodes, do **not** indicate genetic similarity. This information is exclusively determined by the edges.

For example:

...

## Community detection

...

## K-selection plots

...

### Admixture plots

...
