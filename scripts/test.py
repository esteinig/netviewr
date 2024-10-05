import typer
import igraph as ig
import pandas as pd
import matplotlib.pyplot as plt
import matplotlib.colors as mcolors
import random

app = typer.Typer()

@app.command()
def plot_graph(
    edgelist_file: str = typer.Argument(..., help="Path to the edge list file"),
    labels_file: str = typer.Argument(..., help="Path to the CSV file containing labels"),
    label_column: str = typer.Option("label", help="Column name in the CSV for node labels"),
    output_file: str = typer.Option("graph_plot.png", help="File name to save the plot")
):
    """
    Plot a graph using Fruchterman-Reingold layout (force-directed) based on an edge list.
    Nodes will be colored according to the labels specified in a CSV file.
    """
    
    # Read edge list
    edges = pd.read_csv(edgelist_file, sep=" ", header=None)
    g = ig.Graph.TupleList(edges.itertuples(index=False), directed=False)

    # Read labels
    labels_df = pd.read_csv(labels_file)
    
    if label_column not in labels_df.columns:
        typer.echo(f"Error: The label column '{label_column}' is not present in the labels file.")
        raise typer.Exit()

    # Create label dict using the dataframe index as the node index in the graph
    label_dict = labels_df[label_column].to_dict()

    # Ensure the graph nodes have corresponding labels in the label dict
    if max(g.vs.indices) >= len(label_dict):
        typer.echo(f"Error: Node indices in the graph exceed available labels.")
        raise typer.Exit()

    # Assign labels to nodes in the graph by their index
    g.vs["label"] = [label_dict.get(node.index, "Unknown") for node in g.vs]
    
    # Get unique labels
    unique_labels = set(g.vs["label"])

    # Create color mapping for labels
    palette = list(mcolors.TABLEAU_COLORS.values())
    random.shuffle(palette)
    color_map = {label: palette[i % len(palette)] for i, label in enumerate(unique_labels)}

    # Assign colors to nodes
    g.vs["color"] = [color_map[label] for label in g.vs["label"]]

    # Layout the graph with Fruchterman-Reingold
    layout = g.layout("fruchterman_reingold")

    # Plot the graph
    fig, ax = plt.subplots(figsize=(10, 10))
    ig.plot(
        g,
        layout=layout,
        target=ax,
        vertex_color=g.vs["color"],
        vertex_label=None,
        vertex_size=8,
        edge_width=1
    )

    # Add legend for labels
    for label, color in color_map.items():
        ax.plot([], [], color=color, label=label, marker='o', linestyle='None')

    ax.legend(scatterpoints=1, frameon=False, labelspacing=1, loc='best')
    
    # Save the plot
    plt.savefig(output_file, bbox_inches="tight")
    typer.echo(f"Graph plot saved to {output_file}")

if __name__ == "__main__":
    app()
