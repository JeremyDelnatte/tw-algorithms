import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
import glob
import os
import re
import catppuccin
from catppuccin.extras.matplotlib import load_color
# plt.style.use("latte")

# plt.style.use([catppuccin.PALETTE.latte.identifier])

plt.rcParams.update({
    "text.usetex": True,          # use LaTeX
    "font.family": "serif",       # use LaTeX serif font
    "font.serif": ["Computer Modern"],  # default LaTeX font

    # Font sizes
    "axes.labelsize": 14,             # x and y labels
    "axes.titlesize": 16,             # plot title
    "xtick.labelsize": 12,            # x tick labels
    "ytick.labelsize": 12,            # y tick labels
    "legend.fontsize": 12,            # legend

    "figure.facecolor": "white",   # entire figure
    "axes.facecolor": "white",     # plot area
    "savefig.facecolor": "white",  # saved file background
})


# Folder containing CSV files
csv_folder = "benchmarks/instances/dimacs"
csv_files = glob.glob(os.path.join(csv_folder, "*.csv"))

data = []

# Regex to extract algorithm, n, m
pattern = r"(.+)_dimacs\.g6\.csv"

for file in csv_files:
    filename = os.path.basename(file)
    match = re.match(pattern, filename)

    if not match:
        continue

    algorithm, = match.groups()

    df = pd.read_csv(file)

    # Remove graph_g6, timeout columns if they exist
    df = df.drop(columns=[col for col in ["graph_g6", "timeout", "treewidth"] if col in df.columns])

    # Add algorithm column
    df["algorithm"] = algorithm

    data.append(df)

# Create DataFrame
df_all = pd.concat(data, ignore_index=True)

print(df_all)

# Order instances by minimum runtime across algorithms
ordered_instances = (
    df_all.groupby("name")["runtime"]
    .min()
    .sort_values()
    .index
)

print(df_all)
print(ordered_instances)

algorithms = df_all["algorithm"].unique()

# Prepare plot
plt.figure(figsize=(10, 6))

x = np.arange(len(ordered_instances))
width = 0.7 / len(algorithms)

for i, algo in enumerate(["BranchBound", "DynamicProg", "Recursive"]):
    df_algo = (
        df_all[df_all["algorithm"] == algo]
        .set_index("name")
        .reindex(ordered_instances)
        .reset_index()
    )

    color_palette = catppuccin.PALETTE.mocha.identifier
    if i == 0:
        offset = -0.03
        # color = load_color(catppuccin.PALETTE.latte.identifier, "flamingo")
        # color = load_color(catppuccin.PALETTE.latte.identifier, "yellow")
        color = load_color(color_palette, "blue")
    elif i == 2:
        offset = 0.03
        # color = load_color(catppuccin.PALETTE.latte.identifier, "mauve")
        # color = load_color(catppuccin.PALETTE.latte.identifier, "green")
        color = load_color(color_palette, "red")
    else:
        offset = 0.0
        # color = load_color(catppuccin.PALETTE.latte.identifier, "pink")
        # color = load_color(catppuccin.PALETTE.latte.identifier, "teal")
        color = load_color(color_palette, "mauve")

    bars = plt.bar(
        x + i * width + offset,
        df_algo["runtime"],
        width=width,
        linewidth=1.5,
        label=algo,
        color=color,
    )

    # Make fill transparent but edges solid
    for bar in bars:
        r, g, b, _ = bar.get_facecolor()
        bar.set_facecolor((r, g, b, 0.3))   # transparent fill
        bar.set_edgecolor((r, g, b, 1.0))   # solid edge

labels = [f"{n}" for n in ordered_instances]

plt.xticks(x + width * (len(algorithms)-1)/2, labels, rotation=45)
plt.xlabel(r"DIMACS Instance")
plt.ylabel("Runtime $(s)$")
# plt.title(r"Mean Runtime per Graph Size $(\text{ordered by fastest algorithm})$")
plt.legend()
plt.yscale("log")  # Recommended for runtime benchmarks
plt.tight_layout()

plt.savefig("benchmark_plot.svg", format="svg")
plt.show()
