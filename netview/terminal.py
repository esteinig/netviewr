import typer

app = typer.Typer(add_completion=False)


@app.command()
def test():

    """ Terminal application test """

    print("Application test successful")


@app.command()
def plot_graph():
    
    """ Plot a mutual nearest neighbor graph generated with Netview """

    print("Not implemented yet")