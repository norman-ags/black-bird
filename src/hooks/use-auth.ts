import React from "react";
import { AuthContext } from "../provider/AuthProvider";

export const useAuth = () => React.useContext(AuthContext);
