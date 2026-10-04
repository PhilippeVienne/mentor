// Aide pour les tests : la liste des classes d'un élément.
export const classesDe = (element: Element): string[] => element.className.split(/\s+/).filter(Boolean);
