/** Which collapsible parts of the Monitors tab are open. Closed by default. */
export const useMonitorSections = () => {
  let hotkeysOpen = $state(false);
  return {
    get hotkeysOpen() {
      return hotkeysOpen;
    },
    toggleHotkeys: () => {
      hotkeysOpen = !hotkeysOpen;
    },
  };
};
