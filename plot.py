import argparse
import pandas as pd
import matplotlib.pyplot as plt

def main():
    parser = argparse.ArgumentParser(
        description='Process and visualize CSV data.'
    )
    parser.add_argument(
        'filename',
        type=str,
        help='Path to the CSV file'
    )
    parser.add_argument(
        '--output',
        type=str,
        help='Prefix for output plot images',
        default='plot'
    )
    args = parser.parse_args()
    df = pd.read_csv(args.filename)
    metrics = [
        'min',
        'lower_quartile',
        'median',
        'higher_quartile',
        'max',
        'mean',
        'center_x',
        'center_y'
    ]
    for metric in metrics:
        plt.figure()
        plt.plot(df['frame'], df[metric])
        plt.xlabel('frame')
        plt.ylabel(metric)
        plt.title(f"frame vs {metric}")
        plt.grid(True)
        plt.savefig(f"{args.output}_{metric}.png")
        plt.close()

if __name__ == '__main__':
    main()