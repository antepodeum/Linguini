/** @typedef {{download?: boolean, ignored?: boolean, rel?: string | null}} LinkLocalizationAttributes */

export const web = {
  /**
   * @param {string} href
   * @param {LinkLocalizationAttributes} attributes
   * @param {Record<string, unknown>} input
   */
  shouldLocalizeLink(href, attributes, input) {
    void input;
    return href.startsWith("/") && !attributes.download && !attributes.ignored && attributes.rel !== "external";
  },
  /**
   * @param {string} href
   * @param {string} locale
   * @param {Record<string, unknown>} input
   */
  localizeHref(href, locale, input) {
    void input;
    return `/${locale}${href}`;
  },
};
