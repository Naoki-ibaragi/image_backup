import { createContext, useContext, useState } from 'react';

/**
 * NASリストを管理するContext
 */
const NASContext = createContext();

/**
 * NASContextを使用するカスタムフック
 * @returns {Object} nasList, setNasList, inspList, setInspList
 */
export const useNASContext = () => {
  const context = useContext(NASContext);
  if (!context) {
    throw new Error('useNASContext must be used within NASProvider');
  }
  return context;
};

/**
 * NASContextのProvider
 */
export const NASProvider = ({ children }) => {
  const [nasList, setNasList] = useState([]); // LAN上のNAS一覧
  const [inspList, setInspList] = useState([]); // LAN上の外観検査機一覧

  return (
    <NASContext.Provider value={{ nasList, setNasList, inspList, setInspList }}>
      {children}
    </NASContext.Provider>
  );
};
