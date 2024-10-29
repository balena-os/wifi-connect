import { DEFAULT_LANGUAGE, CustomTranslations } from '../config';
import { DefaultTranslations } from '../translations';

function getLocale(): string {
	let locale = '';
	switch (true) {
		case !!window.navigator.language:
			locale = window.navigator.language;
			break;
		default:
			locale = DEFAULT_LANGUAGE;
			break;
	}

	if (locale.length !== 2) {
		locale = locale.substring(0, 2);
	}

	if (locale in DefaultTranslations) {
		return locale;
	} else {
		return DEFAULT_LANGUAGE;
	}
}

function T(stringToTranslate: string): string {
	const locale = getLocale();
	if (locale in CustomTranslations) {
		// @ts-ignore
		if (stringToTranslate in CustomTranslations[locale]) {
			// @ts-ignore
			return CustomTranslations[locale][stringToTranslate];
		}
	}
	if (locale in DefaultTranslations) {
		// @ts-ignore
		if (stringToTranslate in DefaultTranslations[locale]) {
			// @ts-ignore
			return DefaultTranslations[locale][stringToTranslate];
		}
	}
	if (stringToTranslate in CustomTranslations[DEFAULT_LANGUAGE]) {
		// @ts-ignore
		return CustomTranslations[DEFAULT_LANGUAGE][stringToTranslate];
	}
	if (stringToTranslate in DefaultTranslations[DEFAULT_LANGUAGE]) {
		// @ts-ignore
		return DefaultTranslations[DEFAULT_LANGUAGE][stringToTranslate];
	}
	return '';
}

export { T };
