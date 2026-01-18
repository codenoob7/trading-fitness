/**
 * ITH (Investment Time Horizon) calculations.
 */

import { maxDrawdown } from "./metrics";
import type { ExcessGainLossResult } from "./types";

/**
 * Calculate excess gains and losses for ITH epoch detection.
 *
 * @param nav - Array of NAV values
 * @param hurdle - TMAEG hurdle rate (e.g., 0.05 for 5%)
 * @returns ExcessGainLossResult containing excess gains, losses, and ITH epoch information
 */
export function excessGainExcessLoss(
	nav: number[],
	hurdle: number,
): ExcessGainLossResult {
	const n = nav.length;

	if (n === 0) {
		return {
			excessGains: [],
			excessLosses: [],
			numOfIthEpochs: 0,
			ithEpochs: [],
			ithIntervalsCv: Number.NaN,
		};
	}

	const firstNav = nav[0];
	if (firstNav === undefined) {
		return {
			excessGains: [],
			excessLosses: [],
			numOfIthEpochs: 0,
			ithEpochs: [],
			ithIntervalsCv: Number.NaN,
		};
	}

	const excessGains: number[] = new Array(n).fill(0);
	const excessLosses: number[] = new Array(n).fill(0);
	const ithEpochs: boolean[] = new Array(n).fill(false);

	let excessGain = 0;
	let excessLoss = 0;
	let endorsingCrest = firstNav;
	let endorsingNadir = firstNav;
	let candidateCrest = firstNav;
	let candidateNadir = firstNav;

	for (let i = 1; i < n; i++) {
		const equity = nav[i - 1];
		const nextEquity = nav[i];

		if (equity === undefined || nextEquity === undefined) {
			continue;
		}

		if (nextEquity > candidateCrest) {
			excessGain = endorsingCrest !== 0 ? nextEquity / endorsingCrest - 1 : 0;
			candidateCrest = nextEquity;
		}

		if (nextEquity < candidateNadir) {
			excessLoss = 1 - nextEquity / endorsingCrest;
			candidateNadir = nextEquity;
		}

		const resetCondition =
			excessGain > Math.abs(excessLoss) &&
			excessGain > hurdle &&
			candidateCrest >= endorsingCrest;

		if (resetCondition) {
			endorsingCrest = candidateCrest;
			endorsingNadir = equity;
			candidateNadir = equity;
		} else {
			endorsingNadir = Math.min(endorsingNadir, equity);
		}

		excessGains[i] = excessGain;
		excessLosses[i] = excessLoss;

		if (resetCondition) {
			excessGain = 0;
			excessLoss = 0;
		}

		// Check ITH epoch condition
		const excessGainAtI = excessGains[i];
		const excessLossAtI = excessLosses[i];
		if (excessGainAtI !== undefined && excessLossAtI !== undefined) {
			const ithEpochCondition =
				excessGainAtI > excessLossAtI && excessGainAtI > hurdle;
			ithEpochs[i] = ithEpochCondition;
		}
	}

	// Count ITH epochs
	const numOfIthEpochs = ithEpochs.filter((x) => x).length;

	// Calculate ITH intervals CV
	const ithIntervalsCv = calculateIthIntervalsCv(ithEpochs);

	return {
		excessGains,
		excessLosses,
		numOfIthEpochs,
		ithEpochs,
		ithIntervalsCv,
	};
}

/**
 * Calculate coefficient of variation for ITH intervals.
 */
function calculateIthIntervalsCv(ithEpochs: boolean[]): number {
	// Collect epoch indices starting with 0
	const epochIndices: number[] = [0];
	for (let i = 0; i < ithEpochs.length; i++) {
		if (ithEpochs[i]) {
			epochIndices.push(i);
		}
	}

	if (epochIndices.length < 2) {
		return Number.NaN;
	}

	// Calculate intervals
	const intervals: number[] = [];
	for (let i = 1; i < epochIndices.length; i++) {
		const curr = epochIndices[i];
		const prev = epochIndices[i - 1];
		if (curr !== undefined && prev !== undefined) {
			intervals.push(curr - prev);
		}
	}

	if (intervals.length === 0) {
		return Number.NaN;
	}

	// Calculate mean and std
	const n = intervals.length;
	const mean = intervals.reduce((a, b) => a + b, 0) / n;

	if (mean === 0) {
		return Number.NaN;
	}

	const variance = intervals.reduce((sum, x) => sum + (x - mean) ** 2, 0) / n;
	const stdDev = Math.sqrt(variance);

	return stdDev / mean;
}

/**
 * Determine TMAEG from NAV data.
 *
 * @param nav - Array of NAV values
 * @param method - Method to use: "mdd" for max drawdown, "fixed" for fixed value
 * @param fixedValue - Fixed TMAEG value to use if method is "fixed"
 */
export function determineTmaeg(
	nav: number[],
	method: string,
	fixedValue: number,
): number {
	switch (method) {
		case "mdd":
			return maxDrawdown(nav);
		case "fixed":
			return fixedValue;
		default:
			return fixedValue;
	}
}
