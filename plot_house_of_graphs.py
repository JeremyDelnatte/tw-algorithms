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
csv_folder = "benchmarks/instances/house_of_graphs"
csv_files = glob.glob(os.path.join(csv_folder, "*.csv"))

data = []

# Regex to extract algorithm, n, m
pattern = r"(.+)_graphs_n(\d+)\.g6\.csv"

for file in csv_files:
    filename = os.path.basename(file)
    match = re.match(pattern, filename)

    if not match:
        continue

    algorithm, n = match.groups()
    n = int(n)

    df = pd.read_csv(file)

    mean_runtime = df["runtime"].mean()

    data.append({
        "algorithm": algorithm,
        "n": n,
        "mean_runtime": mean_runtime,
        "num_graphs": len(df)
    })

# Create DataFrame
df_all = pd.DataFrame(data)

# Order graph sizes by minimum runtime across algorithms
# ordered_sizes = (
#     df_all.groupby("n")["mean_runtime"]
#     .min()
#     .sort_values()
#     .index
# )

# Order graph sizes by the number of vertices n (ascending)
ordered_sizes = sorted(df_all["n"].unique())

print(df_all)
print(ordered_sizes)

algorithms = df_all["algorithm"].unique()

# Prepare plot
plt.figure(figsize=(10, 6))

x = np.arange(len(ordered_sizes))
width = 0.7 / len(algorithms)

for i, algo in enumerate(["BranchBound", "DynamicProg", "Recursive"]):
    df_algo = (
        df_all[df_all["algorithm"] == algo]
        .set_index("n")
        .reindex(ordered_sizes)
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
        df_algo["mean_runtime"],
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

labels = [f"{n}" for n in ordered_sizes]

plt.xticks(x + width * (len(algorithms)-1)/2, labels)
plt.xlabel(r"Graph Size $(|V|)$")
plt.ylabel("Mean Runtime $(s)$")
# plt.title(r"Mean Runtime per Graph Size $(\text{ordered by fastest algorithm})$")
plt.legend()
plt.yscale("log")  # Recommended for runtime benchmarks
plt.tight_layout()

plt.savefig("benchmark_plot.svg", format="svg")
plt.show()
