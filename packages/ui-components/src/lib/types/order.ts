import type { NameAndDescriptionCfg } from '@rainlanguage/raindex';

export type ValidOrderDetail = {
	details: NameAndDescriptionCfg;
	name: string;
	dotrain: string;
};

export type InvalidOrderDetail = {
	name: string;
	error: string;
};
