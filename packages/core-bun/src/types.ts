/**
 * Type definitions for trading fitness analysis.
 * These types mirror the JSON schemas in shared-types.
 */

import { z } from "zod";

/** NAV record schema */
export const NavRecordSchema = z.object({
	date: z.string(),
	nav: z.number().positive(),
	pnl: z.number().optional(),
});

export type NavRecord = z.infer<typeof NavRecordSchema>;

/** Fitness metrics schema */
export const FitnessMetricsSchema = z.object({
	sharpeRatio: z.number(),
	maxDrawdown: z.number().min(0).max(1),
	totalReturn: z.number(),
	annualizedReturn: z.number().optional(),
	volatility: z.number().min(0).optional(),
	calmarRatio: z.number().optional(),
	sortinoRatio: z.number().optional(),
	tradingDays: z.number().int().positive().optional(),
	marketType: z.enum(["crypto", "stocks", "forex", "futures"]).optional(),
});

export type FitnessMetrics = z.infer<typeof FitnessMetricsSchema>;

/** ITH result schema */
export const IthResultSchema = z.object({
	uid: z.string(),
	sourceFile: z.string().optional(),
	analysisTimestamp: z.string(),
	dateRange: z.object({
		start: z.string(),
		end: z.string(),
		tradingDays: z.number().int().positive(),
	}),
	tmaeg: z.number().min(0).max(1),
	tmaegMethod: z.enum(["mdd", "fixed"]),
	ithEpochs: z.number().int().min(0),
	ithIntervalsCv: z.number(),
	qualified: z.boolean(),
	disqualificationReason: z.string().optional(),
	metrics: FitnessMetricsSchema.optional(),
});

export type IthResult = z.infer<typeof IthResultSchema>;

/** Excess gain/loss result */
export interface ExcessGainLossResult {
	excessGains: number[];
	excessLosses: number[];
	numOfIthEpochs: number;
	ithEpochs: boolean[];
	ithIntervalsCv: number;
}
