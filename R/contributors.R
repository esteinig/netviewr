#'Identification of Key Contributors
#'
#'Identification of key contributors using Horn's Parallel Analysis, Eigenvalue Decomposition and
#'calculation of the Genetic Contribution Score (GCS) as described in Neuditschko et al. (2017).
#'
#'@param matrix                Pairwise relationship / correlation / similarity matrix between samples (N x N)
#'@param paran_iterations      Iterations for computing the number of significant components using Horn's Parallel Analysis in Paran [ int, 100 ]
#'@param paran_centile         Significance level (centile used in estimating bias) in Paran [ int, 99 ]
#'@param dist                  Input matrix is a distance matrix; converted to similarity matrix by subtracting from 1 [ bool, FALSE ]
#'@param verbose               Print computation status and results of Horn's Parallel Analysis [ bool, TRUE ]

#'@usage find_contributors(relMatrix, metaData, paranIterations=100, paranCentile=99, distMatrix=FALSE, verbose=TRUE)
#'
#'@return Data frame containing ordered genetic contribution scores (GCS)
#'
#'@details For examples and tutorials, please see the Repository: \url{https://github.com/esteinig/netviewr}
#'
#'@export
#'@import paran

find_contributors <- function(matrix, paran_iterations=100, paran_centile=99, dist=FALSE, verbose=FALSE){

  require(paran)

  ### Written by Markus Neuditschko and Mehar Khatkar, modified * for use in NetView R by Eike Steinig

  if(dist==TRUE){
    matrix = 1 - matrix
    diag(matrix) = rep(1, nrow(matrix))
  }

  # Set centile, from significance input (as stated in the publication) *

  # Added verbosity *
  if(verbose==FALSE){ quiet = TRUE; stat = FALSE } else { quiet = FALSE; stat = TRUE }

  k = paran(matrix, iterations=paran_iterations, centile=paran_centile, quietly=T, stat=T)

  x = eigen(matrix)

  vectors = x$vectors
  values  = x$values
  n       = length(values)
  D       = matrix(0, nrow =n, ncol=n)
  diag(D) = values

  # Modified access to number of significant - retained PCs by k$Retained *
  k = k$Retained
  RSV   = vector("list",n)
  scores = vector("list",n)

  for (i in 1:n)
  {
    RSV[[i]] = solve(sqrt(D[1:k,1:k])) %*% t(vectors[,1:k]) %*% matrix[,i]
    scores[[i]] = sum(RSV[[i]]^2)
  }

  scores   = as.data.frame (unlist(scores))
  names(scores) = "gcs"

  return(as_tibble(scores))

}
